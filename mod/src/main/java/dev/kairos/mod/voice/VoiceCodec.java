package dev.kairos.mod.voice;

import io.github.jaredmdobson.concentus.OpusApplication;
import io.github.jaredmdobson.concentus.OpusDecoder;
import io.github.jaredmdobson.concentus.OpusEncoder;
import io.github.jaredmdobson.concentus.OpusException;
import io.github.jaredmdobson.concentus.OpusSignal;

/** Squeezes speech into small packets and back: Opus at 16 kHz, mono, in 20 millisecond frames. */
public final class VoiceCodec {
	public static final int SAMPLE_RATE = 16000;
	public static final int FRAME_SAMPLES = 320;
	public static final int FRAME_MILLIS = 20;
	/** The most bytes a single frame may take; the server refuses anything larger. */
	public static final int MAX_PACKET = 400;
	private static final int BITRATE = 24000;

	private VoiceCodec() {
	}

	public static final class Encoder {
		private final OpusEncoder encoder;
		private final byte[] buffer = new byte[MAX_PACKET];

		public Encoder() {
			try {
				encoder = new OpusEncoder(SAMPLE_RATE, 1, OpusApplication.OPUS_APPLICATION_VOIP);
				encoder.setBitrate(BITRATE);
				encoder.setSignalType(OpusSignal.OPUS_SIGNAL_VOICE);
			} catch (OpusException error) {
				throw new IllegalStateException("could not start the voice encoder", error);
			}
		}

		public byte[] encode(short[] frame) {
			if (frame.length != FRAME_SAMPLES) {
				throw new IllegalArgumentException("a frame is " + FRAME_SAMPLES + " samples");
			}
			try {
				int size = encoder.encode(frame, 0, FRAME_SAMPLES, buffer, 0, buffer.length);
				byte[] packet = new byte[size];
				System.arraycopy(buffer, 0, packet, 0, size);
				return packet;
			} catch (OpusException error) {
				throw new IllegalStateException("could not encode a voice frame", error);
			}
		}
	}

	public static final class Decoder {
		private final OpusDecoder decoder;

		public Decoder() {
			try {
				decoder = new OpusDecoder(SAMPLE_RATE, 1);
			} catch (OpusException error) {
				throw new IllegalStateException("could not start the voice decoder", error);
			}
		}

		/** The sound of one packet, or an empty frame when the packet is damaged. */
		public short[] decode(byte[] packet) {
			short[] frame = new short[FRAME_SAMPLES];
			try {
				decoder.decode(packet, 0, packet.length, frame, 0, FRAME_SAMPLES, false);
			} catch (OpusException | RuntimeException error) {
				return new short[FRAME_SAMPLES];
			}
			return frame;
		}
	}
}
