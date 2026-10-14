package dev.kairos.mod.voice;

/** How loud a speaker is for a listener, by the distance between them. */
public final class Attenuation {
	/** Within this many blocks the voice is at full volume. */
	public static final double FULL_VOLUME_RANGE = 8.0;
	/** Beyond this many blocks the voice cannot be heard. */
	public static final double MAX_RANGE = 48.0;

	private Attenuation() {
	}

	public static float gain(double distance) {
		if (distance <= FULL_VOLUME_RANGE) {
			return 1f;
		}
		if (distance >= MAX_RANGE) {
			return 0f;
		}
		double fade = (MAX_RANGE - distance) / (MAX_RANGE - FULL_VOLUME_RANGE);
		return (float) (fade * fade);
	}

	/** Whether the server has a reason to pass a voice frame on to someone at this squared distance. */
	public static boolean isAudible(double distanceSquared) {
		return distanceSquared < MAX_RANGE * MAX_RANGE;
	}
}
