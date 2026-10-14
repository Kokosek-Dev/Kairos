package dev.kairos.mod.client.friends;

import dev.kairos.mod.KairosMod;
import dev.kairos.mod.friends.JoinCode;
import dev.kairos.mod.friends.Upnp;
import java.io.IOException;
import java.net.DatagramSocket;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.util.Optional;
import net.minecraft.client.Minecraft;
import net.minecraft.client.server.IntegratedServer;
import net.minecraft.util.HttpUtil;

/** Opens the world being played to friends and remembers how, so it can be closed again. */
public final class FriendsHost {
	public enum State {
		IDLE, WORKING, OPEN
	}

	private static volatile State state = State.IDLE;
	private static volatile String internetCode;
	private static volatile String networkCode;
	private static volatile String note = "";
	private static Upnp.Gateway gateway;
	private static int mappedPort;

	private FriendsHost() {
	}

	public static State state() {
		return state;
	}

	/** The code for friends on the internet, or null when the router could not be asked to let them in. */
	public static String internetCode() {
		return internetCode;
	}

	/** The code for friends on the same network. */
	public static String networkCode() {
		return networkCode;
	}

	/** A plain-language remark about how it went, as a translation key and its arguments. */
	public static String note() {
		return note;
	}

	public static boolean canOpen(Minecraft minecraft) {
		IntegratedServer server = minecraft.getSingleplayerServer();
		return server != null && !server.isPublished() && state == State.IDLE;
	}

	public static void open(Minecraft minecraft) {
		IntegratedServer server = minecraft.getSingleplayerServer();
		if (server == null || server.isPublished() || state != State.IDLE) {
			return;
		}
		int port = HttpUtil.getAvailablePort();
		if (!server.publishServer(server.getDefaultGameType(), false, port)) {
			note = "kairos.friends.failed";
			return;
		}
		state = State.WORKING;
		note = "kairos.friends.asking";
		Thread worker = new Thread(() -> reach(port), "Kairos-Friends");
		worker.setDaemon(true);
		worker.start();
	}

	private static void reach(int port) {
		try {
			Optional<String> local = localAddress();
			local.ifPresent(address -> networkCode = encode(address, port));
			Optional<Upnp.Gateway> found = Upnp.discover();
			if (found.isEmpty()) {
				note = "kairos.friends.no_router";
				return;
			}
			Optional<String> external = Upnp.externalAddress(found.get());
			if (external.isEmpty() || !Upnp.isReachableFromInternet(InetAddress.getByName(external.get()))) {
				note = "kairos.friends.shared_address";
				return;
			}
			Upnp.open(found.get(), port, local.orElseThrow(() -> new IOException("no local address")), "Kairos");
			gateway = found.get();
			mappedPort = port;
			internetCode = encode(external.get(), port);
			note = "kairos.friends.ready";
		} catch (IOException | RuntimeException error) {
			KairosMod.LOGGER.warn("Could not open the world to the internet: {}", error.toString());
			note = "kairos.friends.router_refused";
		} finally {
			state = State.OPEN;
		}
	}

	/** Takes the port away from the router again. Called when the world closes. */
	public static void close() {
		Upnp.Gateway toClose = gateway;
		int port = mappedPort;
		gateway = null;
		mappedPort = 0;
		state = State.IDLE;
		internetCode = null;
		networkCode = null;
		note = "";
		if (toClose != null) {
			Thread worker = new Thread(() -> {
				try {
					Upnp.close(toClose, port);
				} catch (IOException error) {
					KairosMod.LOGGER.warn("Could not close the port again: {}", error.toString());
				}
			}, "Kairos-Friends-Close");
			worker.setDaemon(true);
			worker.start();
		}
	}

	private static String encode(String address, int port) {
		try {
			return JoinCode.encode(InetAddress.getByName(address), port);
		} catch (IOException error) {
			throw new IllegalStateException(error);
		}
	}

	/** The address this computer has on the local network; connecting a datagram socket sends nothing. */
	private static Optional<String> localAddress() {
		try (DatagramSocket socket = new DatagramSocket()) {
			socket.connect(new InetSocketAddress("192.0.2.1", 9));
			return Optional.of(socket.getLocalAddress().getHostAddress());
		} catch (IOException error) {
			return Optional.empty();
		}
	}
}
