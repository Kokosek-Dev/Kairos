package dev.kairos.mod.voice;

import dev.kairos.mod.net.VoiceDown;
import dev.kairos.mod.net.VoiceUp;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import net.fabricmc.fabric.api.networking.v1.PayloadTypeRegistry;
import net.fabricmc.fabric.api.networking.v1.ServerPlayConnectionEvents;
import net.fabricmc.fabric.api.networking.v1.ServerPlayNetworking;
import net.minecraft.server.level.ServerPlayer;

/** Passes each player's voice on to the players near enough to hear it. */
public final class VoiceServer {
	/** A real voice needs 50 frames a second; anything far above that is not a voice. */
	static final int MAX_FRAMES_PER_SECOND = 80;
	private static final long SECOND_NANOS = 1_000_000_000L;

	private static final Map<UUID, Window> WINDOWS = new ConcurrentHashMap<>();

	private VoiceServer() {
	}

	/** Counts frames in the current second for one player. */
	static final class Window {
		private long start;
		private int frames;

		synchronized boolean allow(long now) {
			if (now - start >= SECOND_NANOS) {
				start = now;
				frames = 0;
			}
			return ++frames <= MAX_FRAMES_PER_SECOND;
		}
	}

	public static void register() {
		PayloadTypeRegistry.playC2S().register(VoiceUp.TYPE, VoiceUp.CODEC);
		PayloadTypeRegistry.playS2C().register(VoiceDown.TYPE, VoiceDown.CODEC);
		ServerPlayNetworking.registerGlobalReceiver(VoiceUp.TYPE, (payload, context) -> relay(context.player(), payload));
		ServerPlayConnectionEvents.DISCONNECT.register((handler, server) -> WINDOWS.remove(handler.getPlayer().getUUID()));
	}

	static boolean acceptable(byte[] data) {
		return data.length > 0 && data.length <= VoiceCodec.MAX_PACKET;
	}

	private static void relay(ServerPlayer speaker, VoiceUp frame) {
		if (!acceptable(frame.data()) || !WINDOWS.computeIfAbsent(speaker.getUUID(), id -> new Window()).allow(System.nanoTime())) {
			return;
		}
		VoiceDown outgoing = new VoiceDown(speaker.getUUID(), frame.sequence(), frame.data());
		for (ServerPlayer listener : speaker.server.getPlayerList().getPlayers()) {
			if (listener != speaker
				&& listener.level() == speaker.level()
				&& Attenuation.isAudible(listener.distanceToSqr(speaker))
				&& ServerPlayNetworking.canSend(listener, VoiceDown.TYPE)) {
				ServerPlayNetworking.send(listener, outgoing);
			}
		}
	}
}
