package dev.kairos.mod.friends;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.net.InetAddress;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.util.Optional;
import org.junit.jupiter.api.Test;

class UpnpTest {
	private static final String DESCRIPTION = """
		<?xml version="1.0"?>
		<root xmlns="urn:schemas-upnp-org:device-1-0"><device><deviceList><device><deviceList><device><serviceList>
		<service><serviceType>urn:schemas-upnp-org:service:WANCommonInterfaceConfig:1</serviceType><controlURL>/ctl/common</controlURL></service>
		<service><serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType><controlURL>/ctl/IPConn</controlURL></service>
		</serviceList></device></deviceList></device></deviceList></device></root>
		""";

	@Test
	void findsTheConnectionServiceOfARouterOnTheLocalNetwork() throws Exception {
		Optional<Upnp.Gateway> gateway = Upnp.parseGateway(DESCRIPTION.getBytes(StandardCharsets.UTF_8), URI.create("http://192.168.1.1:5000/rootDesc.xml").toURL());

		assertTrue(gateway.isPresent());
		assertEquals("http://192.168.1.1:5000/ctl/IPConn", gateway.get().controlUrl().toString());
		assertEquals("urn:schemas-upnp-org:service:WANIPConnection:1", gateway.get().serviceType());
	}

	@Test
	void ignoresADescriptionWhoseServiceLivesOutsideTheLocalNetwork() throws Exception {
		String foreign = DESCRIPTION.replace("/ctl/IPConn", "http://203.0.113.9/ctl/IPConn");

		assertTrue(Upnp.parseGateway(foreign.getBytes(StandardCharsets.UTF_8), URI.create("http://192.168.1.1:5000/rootDesc.xml").toURL()).isEmpty());
	}

	@Test
	void refusesXmlThatDeclaresEntities() throws Exception {
		String attack = "<?xml version=\"1.0\"?><!DOCTYPE root [<!ENTITY x SYSTEM \"file:///C:/Windows/win.ini\">]><root>&x;</root>";

		assertTrue(Upnp.parseGateway(attack.getBytes(StandardCharsets.UTF_8), URI.create("http://192.168.1.1/d.xml").toURL()).isEmpty());
		assertEquals(null, Upnp.tag(attack, "root"));
	}

	@Test
	void readsTheLocationFromARouterReply() {
		String reply = "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=120\r\nLocation: http://192.168.1.1:5000/rootDesc.xml\r\nST: upnp:rootdevice\r\n\r\n";

		assertEquals(Optional.of("http://192.168.1.1:5000/rootDesc.xml"), Upnp.location(reply));
		assertEquals(Optional.empty(), Upnp.location("HTTP/1.1 200 OK\r\n\r\n"));
	}

	@Test
	void readsTheExternalAddressFromAnswer() {
		String answer = "<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\"><s:Body><u:GetExternalIPAddressResponse xmlns:u=\"urn:x\">"
			+ "<NewExternalIPAddress>203.0.113.42</NewExternalIPAddress></u:GetExternalIPAddressResponse></s:Body></s:Envelope>";

		assertEquals("203.0.113.42", Upnp.tag(answer, "NewExternalIPAddress"));
	}

	@Test
	void escapesTextPutIntoTheRequest() {
		assertEquals("a&amp;b &lt;c&gt; &quot;d&quot;", Upnp.escape("a&b <c> \"d\""));
		assertTrue(Upnp.envelope("urn:x", "AddPortMapping", "<A>1</A>").contains("<u:AddPortMapping xmlns:u=\"urn:x\"><A>1</A></u:AddPortMapping>"));
	}

	@Test
	void knowsWhichAddressesFriendsCannotReach() throws Exception {
		assertTrue(Upnp.isReachableFromInternet(InetAddress.getByName("203.0.113.42")));
		for (String shared : new String[] {"100.64.0.5", "100.127.255.1", "10.0.0.2", "192.168.1.20", "172.16.4.4", "127.0.0.1", "169.254.1.1"}) {
			assertFalse(Upnp.isReachableFromInternet(InetAddress.getByName(shared)), shared);
		}
		assertTrue(Upnp.isReachableFromInternet(InetAddress.getByName("100.128.0.1")));
	}
}
