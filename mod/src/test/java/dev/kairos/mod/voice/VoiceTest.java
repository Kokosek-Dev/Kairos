package dev.kairos.mod.voice;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class VoiceTest {
	private static short[] tone(double frequency, double amplitude, int offsetSamples) {
		short[] frame = new short[VoiceCodec.FRAME_SAMPLES];
		for (int index = 0; index < frame.length; index++) {
			frame[index] = (short) (amplitude * Math.sin(2 * Math.PI * frequency * (index + offsetSamples) / VoiceCodec.SAMPLE_RATE));
		}
		return frame;
	}

	private static double energy(short[] frame) {
		double sum = 0;
		for (short sample : frame) {
			sum += (double) sample * sample;
		}
		return Math.sqrt(sum / frame.length);
	}

	@Test
	void aVoiceFrameSurvivesTheRoundTripAndGetsMuchSmaller() {
		VoiceCodec.Encoder encoder = new VoiceCodec.Encoder();
		VoiceCodec.Decoder decoder = new VoiceCodec.Decoder();
		short[] last = null;
		int packetSize = 0;
		for (int frame = 0; frame < 20; frame++) {
			byte[] packet = encoder.encode(tone(440, 8000, frame * VoiceCodec.FRAME_SAMPLES));
			packetSize = packet.length;
			last = decoder.decode(packet);
		}

		assertTrue(packetSize > 0 && packetSize < VoiceCodec.FRAME_SAMPLES * 2 / 4, "a frame of " + VoiceCodec.FRAME_SAMPLES * 2 + " bytes became " + packetSize);
		double original = energy(tone(440, 8000, 0));
		double decoded = energy(last);
		assertTrue(decoded > original * 0.5 && decoded < original * 1.5, "loudness " + decoded + " against " + original);
	}

	@Test
	void silenceStaysQuiet() {
		VoiceCodec.Encoder encoder = new VoiceCodec.Encoder();
		VoiceCodec.Decoder decoder = new VoiceCodec.Decoder();
		short[] last = null;
		for (int frame = 0; frame < 10; frame++) {
			last = decoder.decode(encoder.encode(new short[VoiceCodec.FRAME_SAMPLES]));
		}
		assertTrue(energy(last) < 50, "energy " + energy(last));
	}

	@Test
	void damagedPacketsBecomeSilenceInsteadOfCrashing() {
		VoiceCodec.Decoder decoder = new VoiceCodec.Decoder();

		assertEquals(0, energy(decoder.decode(new byte[0])), 0.0);
		assertEquals(VoiceCodec.FRAME_SAMPLES, decoder.decode(new byte[] {127, 1, 2, 3}).length);
	}

	@Test
	void refusesAFrameOfTheWrongSize() {
		assertThrows(IllegalArgumentException.class, () -> new VoiceCodec.Encoder().encode(new short[10]));
	}

	@Test
	void voiceFadesWithDistance() {
		assertEquals(1f, Attenuation.gain(0));
		assertEquals(1f, Attenuation.gain(Attenuation.FULL_VOLUME_RANGE));
		assertEquals(0f, Attenuation.gain(Attenuation.MAX_RANGE));
		assertEquals(0f, Attenuation.gain(500));
		assertTrue(Attenuation.gain(20) > Attenuation.gain(30));
		assertTrue(Attenuation.gain(30) > 0f);
	}

	@Test
	void theServerOnlyPassesOnFramesWithinTheRange() {
		assertTrue(Attenuation.isAudible(47 * 47));
		assertFalse(Attenuation.isAudible(49 * 49));
	}

	@Test
	void theServerRefusesOversizedOrEmptyFrames() {
		assertFalse(VoiceServer.acceptable(new byte[0]));
		assertTrue(VoiceServer.acceptable(new byte[VoiceCodec.MAX_PACKET]));
		assertFalse(VoiceServer.acceptable(new byte[VoiceCodec.MAX_PACKET + 1]));
	}

	@Test
	void aPlayerCannotSendMoreThanAVoiceNeeds() {
		VoiceServer.Window window = new VoiceServer.Window();
		long start = 10_000_000_000L;
		int accepted = 0;
		for (int index = 0; index < 500; index++) {
			if (window.allow(start + index * 1_000L)) {
				accepted++;
			}
		}
		assertEquals(VoiceServer.MAX_FRAMES_PER_SECOND, accepted);
		assertTrue(window.allow(start + 2_000_000_000L), "a new second starts fresh");
	}
}
