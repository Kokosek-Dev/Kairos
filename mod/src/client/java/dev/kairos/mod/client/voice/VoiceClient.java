package dev.kairos.mod.client.voice;

import dev.kairos.mod.KairosMod;
import dev.kairos.mod.net.VoiceDown;
import dev.kairos.mod.net.VoiceUp;
import dev.kairos.mod.voice.Attenuation;
import dev.kairos.mod.voice.VoiceCodec;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentLinkedQueue;
import javax.sound.sampled.AudioFormat;
import javax.sound.sampled.AudioSystem;
import javax.sound.sampled.DataLine;
import javax.sound.sampled.LineUnavailableException;
import javax.sound.sampled.Mixer;
import javax.sound.sampled.SourceDataLine;
import javax.sound.sampled.TargetDataLine;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.world.entity.player.Player;

/** Listens to the microphone, sends speech to the server, and plays back the players around you. */
public final class VoiceClient {
	private static final AudioFormat MIC_FORMAT = new AudioFormat(VoiceCodec.SAMPLE_RATE, 16, 1, true, false);
	private static final AudioFormat OUTPUT_FORMAT = new AudioFormat(VoiceCodec.SAMPLE_RATE, 16, 2, true, false);
	private static final int HANGOVER_FRAMES = 12;
	/** Frames that may wait for one speaker before the oldest are dropped, which keeps the delay short. */
	private static final int MAX_QUEUED = 6;
	private static final long RECENT_NANOS = 400_000_000L;
	private static final float SIDE_STRENGTH = 0.7f;
	private static final float UNSEEN_GAIN = 0.4f;

	private static final Map<UUID, Speaker> SPEAKERS = new ConcurrentHashMap<>();
	private static volatile boolean running;
	private static volatile boolean pushed;
	private static volatile boolean transmitting;
	private static volatile float level;
	private static volatile String problem = "";
	private static TargetDataLine mic;
	private static SourceDataLine output;

	private VoiceClient() {
	}

	private static final class Speaker {
		final UUID id;
		final VoiceCodec.Decoder decoder = new VoiceCodec.Decoder();
		final ConcurrentLinkedQueue<byte[]> queue = new ConcurrentLinkedQueue<>();
		volatile float left = 1f;
		volatile float right = 1f;
		volatile long lastHeard;

		Speaker(UUID id) {
			this.id = id;
		}

		short[] nextFrame() {
			while (queue.size() > MAX_QUEUED) {
				queue.poll();
			}
			byte[] packet = queue.poll();
			return packet == null ? null : decoder.decode(packet);
		}
	}

	public static boolean isRunning() {
		return running;
	}

	public static boolean isTransmitting() {
		return transmitting;
	}

	/** How loud the microphone is right now, from 0 to 1. */
	public static float level() {
		return level;
	}

	/** Why the microphone is not working, or an empty text. */
	public static String problem() {
		return problem;
	}

	public static void setPushed(boolean down) {
		pushed = down;
	}

	/** Names of the players heard a moment ago. */
	public static List<String> speaking() {
		long now = System.nanoTime();
		Minecraft minecraft = Minecraft.getInstance();
		List<String> names = new ArrayList<>();
		for (Speaker speaker : SPEAKERS.values()) {
			if (now - speaker.lastHeard < RECENT_NANOS && minecraft.getConnection() != null) {
				var info = minecraft.getConnection().getPlayerInfo(speaker.id);
				names.add(info == null ? "?" : info.getProfile().getName());
			}
		}
		return names;
	}

	/** The microphones this computer offers, by name. */
	public static List<String> inputDevices() {
		List<String> names = new ArrayList<>();
		DataLine.Info wanted = new DataLine.Info(TargetDataLine.class, MIC_FORMAT);
		for (Mixer.Info info : AudioSystem.getMixerInfo()) {
			if (AudioSystem.getMixer(info).isLineSupported(wanted)) {
				names.add(info.getName());
			}
		}
		return names;
	}

	public static synchronized void start() {
		if (running) {
			return;
		}
		running = true;
		problem = "";
		VoiceSettings settings = VoiceSettings.get();
		try {
			output = (SourceDataLine) AudioSystem.getLine(new DataLine.Info(SourceDataLine.class, OUTPUT_FORMAT));
			output.open(OUTPUT_FORMAT, VoiceCodec.FRAME_SAMPLES * 4 * 3);
			output.start();
			daemon("Kairos-Voice-Out", VoiceClient::playbackLoop);
		} catch (LineUnavailableException | RuntimeException error) {
			KairosMod.LOGGER.warn("No sound output for voice chat: {}", error.toString());
			problem = "kairos.voice.no_output";
		}
		try {
			mic = openMic(settings.inputDevice);
			mic.open(MIC_FORMAT, VoiceCodec.FRAME_SAMPLES * 2 * 4);
			mic.start();
			daemon("Kairos-Voice-In", VoiceClient::captureLoop);
		} catch (LineUnavailableException | RuntimeException error) {
			KairosMod.LOGGER.warn("No microphone for voice chat: {}", error.toString());
			problem = "kairos.voice.no_mic";
		}
	}

	public static synchronized void stop() {
		running = false;
		transmitting = false;
		level = 0f;
		SPEAKERS.clear();
		close(mic);
		close(output);
		mic = null;
		output = null;
	}

	private static void close(DataLine line) {
		if (line != null) {
			line.stop();
			line.close();
		}
	}

	private static void daemon(String name, Runnable body) {
		Thread thread = new Thread(body, name);
		thread.setDaemon(true);
		thread.start();
	}

	private static TargetDataLine openMic(String name) throws LineUnavailableException {
		DataLine.Info info = new DataLine.Info(TargetDataLine.class, MIC_FORMAT);
		if (!name.isEmpty()) {
			for (Mixer.Info candidate : AudioSystem.getMixerInfo()) {
				if (candidate.getName().equals(name) && AudioSystem.getMixer(candidate).isLineSupported(info)) {
					return (TargetDataLine) AudioSystem.getMixer(candidate).getLine(info);
				}
			}
		}
		return (TargetDataLine) AudioSystem.getLine(info);
	}

	private static void captureLoop() {
		VoiceCodec.Encoder encoder = new VoiceCodec.Encoder();
		byte[] raw = new byte[VoiceCodec.FRAME_SAMPLES * 2];
		short[] frame = new short[VoiceCodec.FRAME_SAMPLES];
		int hangover = 0;
		int sequence = 0;
		TargetDataLine line = mic;
		while (running && line != null && line.isOpen()) {
			int filled = 0;
			while (filled < raw.length && running) {
				int read = line.read(raw, filled, raw.length - filled);
				if (read <= 0) {
					return;
				}
				filled += read;
			}
			ByteBuffer.wrap(raw).order(ByteOrder.LITTLE_ENDIAN).asShortBuffer().get(frame);
			double loudness = rms(frame);
			level = (float) Math.min(1.0, loudness / 6000.0);

			VoiceSettings settings = VoiceSettings.get();
			boolean speak;
			if (!settings.enabled) {
				speak = false;
			} else if (settings.pushToTalk) {
				speak = pushed;
			} else {
				if (loudness > settings.openMicThreshold()) {
					hangover = HANGOVER_FRAMES;
				}
				speak = hangover-- > 0;
			}
			transmitting = speak;
			if (speak) {
				VoiceUp packet = new VoiceUp(sequence++, encoder.encode(frame));
				Minecraft.getInstance().execute(() -> {
					if (ClientPlayNetworking.canSend(VoiceUp.TYPE)) {
						ClientPlayNetworking.send(packet);
					}
				});
			}
		}
	}

	static double rms(short[] frame) {
		double sum = 0;
		for (short sample : frame) {
			sum += (double) sample * sample;
		}
		return Math.sqrt(sum / frame.length);
	}

	private static void playbackLoop() {
		byte[] out = new byte[VoiceCodec.FRAME_SAMPLES * 4];
		int[] mix = new int[VoiceCodec.FRAME_SAMPLES * 2];
		SourceDataLine line = output;
		while (running && line != null && line.isOpen()) {
			Arrays.fill(mix, 0);
			for (Speaker speaker : SPEAKERS.values()) {
				short[] frame = speaker.nextFrame();
				if (frame != null) {
					float left = speaker.left;
					float right = speaker.right;
					for (int index = 0; index < frame.length; index++) {
						mix[index * 2] += (int) (frame[index] * left);
						mix[index * 2 + 1] += (int) (frame[index] * right);
					}
				}
			}
			ByteBuffer buffer = ByteBuffer.wrap(out).order(ByteOrder.LITTLE_ENDIAN);
			for (int value : mix) {
				buffer.putShort((short) Math.max(Short.MIN_VALUE, Math.min(Short.MAX_VALUE, value)));
			}
			line.write(out, 0, out.length);
		}
	}

	/** Called for every voice frame the server passes on. */
	public static void receive(VoiceDown frame) {
		if (!running) {
			return;
		}
		Speaker speaker = SPEAKERS.computeIfAbsent(frame.speaker(), Speaker::new);
		speaker.lastHeard = System.nanoTime();
		speaker.queue.add(frame.data());
	}

	/** Called every game tick: works out how loud and from which side each player is heard. */
	public static void tick(Minecraft minecraft) {
		LocalPlayer me = minecraft.player;
		ClientLevel world = minecraft.level;
		if (!running || me == null || world == null) {
			return;
		}
		float master = VoiceSettings.get().volume / 100f;
		double yaw = Math.toRadians(me.getYRot());
		double rightX = -Math.cos(yaw);
		double rightZ = -Math.sin(yaw);
		for (Speaker speaker : SPEAKERS.values()) {
			Player other = world.getPlayerByUUID(speaker.id);
			float gain = UNSEEN_GAIN;
			float pan = 0f;
			if (other != null) {
				gain = Attenuation.gain(other.distanceTo(me));
				double dx = other.getX() - me.getX();
				double dz = other.getZ() - me.getZ();
				double length = Math.hypot(dx, dz);
				pan = length < 1e-3 ? 0f : (float) ((dx * rightX + dz * rightZ) / length);
			}
			speaker.left = gain * master * (1f - Math.max(0f, pan) * SIDE_STRENGTH);
			speaker.right = gain * master * (1f + Math.min(0f, pan) * SIDE_STRENGTH);
		}
	}
}
