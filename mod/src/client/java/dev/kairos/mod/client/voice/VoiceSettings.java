package dev.kairos.mod.client.voice;

import com.google.gson.Gson;
import dev.kairos.mod.KairosMod;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import net.fabricmc.loader.api.FabricLoader;

/** The player's voice choices, kept in the game's config folder. */
public final class VoiceSettings {
	private static final Gson GSON = new Gson();
	private static VoiceSettings current;

	public boolean enabled = true;
	public boolean pushToTalk = true;
	/** The name of the microphone to use; empty means the system's default. */
	public String inputDevice = "";
	/** How loud other players are, in percent. */
	public int volume = 100;
	/** How easily open mic starts to send, from 0 (only loud) to 100 (even whispers). */
	public int sensitivity = 50;

	private static Path file() {
		return FabricLoader.getInstance().getConfigDir().resolve("kairos-voice.json");
	}

	public static synchronized VoiceSettings get() {
		if (current == null) {
			try {
				current = GSON.fromJson(Files.readString(file()), VoiceSettings.class);
			} catch (IOException | RuntimeException error) {
				current = null;
			}
			if (current == null) {
				current = new VoiceSettings();
			}
			current.volume = Math.max(0, Math.min(200, current.volume));
			current.sensitivity = Math.max(0, Math.min(100, current.sensitivity));
			if (current.inputDevice == null) {
				current.inputDevice = "";
			}
		}
		return current;
	}

	public void save() {
		try {
			Files.writeString(file(), GSON.toJson(this));
		} catch (IOException error) {
			KairosMod.LOGGER.warn("Could not save the voice settings: {}", error.toString());
		}
	}

	/** The loudness an open microphone has to reach before it starts sending. */
	public int openMicThreshold() {
		return 1500 - 14 * sensitivity;
	}
}
