package dev.kairos.mod.friends;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.DatagramPacket;
import java.net.DatagramSocket;
import java.net.HttpURLConnection;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.net.URI;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.Locale;
import java.util.Optional;
import javax.xml.XMLConstants;
import javax.xml.parsers.DocumentBuilderFactory;
import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.NodeList;

/**
 * Asks the home router to open a port (UPnP), the way many games do, so friends can reach a world on this computer.
 * Only devices on the local network are talked to, and nothing is kept on the router once the mapping is removed.
 */
public final class Upnp {
	private static final String SEARCH = "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\n"
		+ "ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\r\n";
	private static final String[] SERVICES = {
		"urn:schemas-upnp-org:service:WANIPConnection:2",
		"urn:schemas-upnp-org:service:WANIPConnection:1",
		"urn:schemas-upnp-org:service:WANPPPConnection:1",
	};
	private static final int TIMEOUT_MS = 3000;
	private static final int MAX_BYTES = 256 * 1024;

	/** The router's control address and the kind of service it offers. */
	public record Gateway(URL controlUrl, String serviceType) {
	}

	private Upnp() {
	}

	/** Finds the router on the local network, or empty when none answers. */
	public static Optional<Gateway> discover() {
		try (DatagramSocket socket = new DatagramSocket()) {
			socket.setSoTimeout(TIMEOUT_MS);
			byte[] request = SEARCH.getBytes(StandardCharsets.US_ASCII);
			socket.send(new DatagramPacket(request, request.length, new InetSocketAddress("239.255.255.250", 1900)));
			long deadline = System.currentTimeMillis() + TIMEOUT_MS;
			byte[] buffer = new byte[2048];
			while (System.currentTimeMillis() < deadline) {
				DatagramPacket reply = new DatagramPacket(buffer, buffer.length);
				socket.receive(reply);
				Optional<String> location = location(new String(reply.getData(), 0, reply.getLength(), StandardCharsets.US_ASCII));
				if (location.isPresent()) {
					Optional<Gateway> gateway = describe(location.get());
					if (gateway.isPresent()) {
						return gateway;
					}
				}
			}
		} catch (IOException ignored) {
			// No router answered in time, or the network does not allow it.
		}
		return Optional.empty();
	}

	static Optional<String> location(String reply) {
		for (String line : reply.split("\r?\n")) {
			if (line.toLowerCase(Locale.ROOT).startsWith("location:")) {
				return Optional.of(line.substring("location:".length()).trim());
			}
		}
		return Optional.empty();
	}

	private static Optional<Gateway> describe(String location) {
		try {
			URL url = URI.create(location).toURL();
			if (!isLocalNetwork(InetAddress.getByName(url.getHost()))) {
				return Optional.empty();
			}
			return parseGateway(read(url), url);
		} catch (IOException | IllegalArgumentException error) {
			return Optional.empty();
		}
	}

	/** A reply that points anywhere but this network is not a router, so it is ignored. */
	static boolean isLocalNetwork(InetAddress address) {
		return address.isSiteLocalAddress() || address.isLinkLocalAddress();
	}

	static Optional<Gateway> parseGateway(byte[] xml, URL base) {
		try {
			Document document = parse(xml);
			NodeList services = document.getElementsByTagName("service");
			for (String wanted : SERVICES) {
				for (int index = 0; index < services.getLength(); index++) {
					Element service = (Element) services.item(index);
					if (wanted.equals(text(service, "serviceType")) && text(service, "controlURL") != null) {
						URL control = base.toURI().resolve(text(service, "controlURL")).toURL();
						if (isLocalNetwork(InetAddress.getByName(control.getHost()))) {
							return Optional.of(new Gateway(control, wanted));
						}
					}
				}
			}
		} catch (Exception ignored) {
			// A description that cannot be read is treated as "not a router".
		}
		return Optional.empty();
	}

	public static Optional<String> externalAddress(Gateway gateway) throws IOException {
		return Optional.ofNullable(tag(call(gateway, "GetExternalIPAddress", ""), "NewExternalIPAddress")).filter(address -> !address.isBlank());
	}

	public static void open(Gateway gateway, int port, String localAddress, String description) throws IOException {
		call(gateway, "AddPortMapping", "<NewRemoteHost></NewRemoteHost><NewExternalPort>" + port + "</NewExternalPort>"
			+ "<NewProtocol>TCP</NewProtocol><NewInternalPort>" + port + "</NewInternalPort>"
			+ "<NewInternalClient>" + escape(localAddress) + "</NewInternalClient><NewEnabled>1</NewEnabled>"
			+ "<NewPortMappingDescription>" + escape(description) + "</NewPortMappingDescription><NewLeaseDuration>0</NewLeaseDuration>");
	}

	public static void close(Gateway gateway, int port) throws IOException {
		call(gateway, "DeletePortMapping", "<NewRemoteHost></NewRemoteHost><NewExternalPort>" + port + "</NewExternalPort><NewProtocol>TCP</NewProtocol>");
	}

	/** An address that cannot be reached from the internet, such as one shared by a whole provider (CGNAT). */
	public static boolean isReachableFromInternet(InetAddress address) {
		byte[] bytes = address.getAddress();
		boolean sharedByProvider = bytes.length == 4 && (bytes[0] & 0xff) == 100 && (bytes[1] & 0xc0) == 64;
		return !(sharedByProvider || address.isSiteLocalAddress() || address.isLoopbackAddress() || address.isLinkLocalAddress() || address.isAnyLocalAddress());
	}

	static String envelope(String serviceType, String action, String arguments) {
		return "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" "
			+ "s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body><u:" + action + " xmlns:u=\"" + serviceType + "\">"
			+ arguments + "</u:" + action + "></s:Body></s:Envelope>";
	}

	static String escape(String value) {
		return value.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\"", "&quot;");
	}

	static String tag(String xml, String name) {
		try {
			NodeList nodes = parse(xml.getBytes(StandardCharsets.UTF_8)).getElementsByTagName(name);
			return nodes.getLength() == 0 ? null : nodes.item(0).getTextContent();
		} catch (Exception error) {
			return null;
		}
	}

	private static String call(Gateway gateway, String action, String arguments) throws IOException {
		byte[] body = envelope(gateway.serviceType(), action, arguments).getBytes(StandardCharsets.UTF_8);
		HttpURLConnection connection = (HttpURLConnection) gateway.controlUrl().openConnection();
		connection.setConnectTimeout(TIMEOUT_MS);
		connection.setReadTimeout(TIMEOUT_MS);
		connection.setRequestMethod("POST");
		connection.setDoOutput(true);
		connection.setRequestProperty("Content-Type", "text/xml; charset=\"utf-8\"");
		connection.setRequestProperty("SOAPAction", "\"" + gateway.serviceType() + "#" + action + "\"");
		connection.getOutputStream().write(body);
		if (connection.getResponseCode() >= 400) {
			throw new IOException("the router refused " + action + " (" + connection.getResponseCode() + ")");
		}
		return new String(readLimited(connection.getInputStream()), StandardCharsets.UTF_8);
	}

	private static byte[] read(URL url) throws IOException {
		HttpURLConnection connection = (HttpURLConnection) url.openConnection();
		connection.setConnectTimeout(TIMEOUT_MS);
		connection.setReadTimeout(TIMEOUT_MS);
		return readLimited(connection.getInputStream());
	}

	private static byte[] readLimited(InputStream stream) throws IOException {
		try (stream) {
			byte[] bytes = stream.readNBytes(MAX_BYTES + 1);
			if (bytes.length > MAX_BYTES) {
				throw new IOException("the router sent too much");
			}
			return bytes;
		}
	}

	private static Document parse(byte[] xml) throws Exception {
		DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
		factory.setFeature("http://apache.org/xml/features/disallow-doctype-decl", true);
		factory.setFeature(XMLConstants.FEATURE_SECURE_PROCESSING, true);
		factory.setXIncludeAware(false);
		factory.setExpandEntityReferences(false);
		return factory.newDocumentBuilder().parse(new ByteArrayInputStream(xml));
	}

	private static String text(Element parent, String name) {
		NodeList nodes = parent.getElementsByTagName(name);
		return nodes.getLength() == 0 ? null : nodes.item(0).getTextContent().trim();
	}
}
