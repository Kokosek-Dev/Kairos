package dev.kairos.mod;

import dev.kairos.mod.voice.VoiceServer;
import net.fabricmc.api.ModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class KairosMod implements ModInitializer {
	public static final String ID = "kairos";
	public static final Logger LOGGER = LoggerFactory.getLogger("Kairos");

	@Override
	public void onInitialize() {
		VoiceServer.register();
		LOGGER.info("Kairos is here.");
	}
}
