package dev.kairos.mod.client.voice;

import dev.kairos.mod.client.ui.Theme;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/** A small note in the corner of the screen: when you are being heard, and who is talking to you. */
public final class VoiceHud {
	private static final int MARGIN = 6;
	private static final int LINE = 11;

	private VoiceHud() {
	}

	public static void render(GuiGraphics graphics) {
		Minecraft minecraft = Minecraft.getInstance();
		if (!VoiceClient.isRunning() || minecraft.options.hideGui || minecraft.screen != null) {
			return;
		}
		int y = MARGIN;
		if (!VoiceSettings.get().enabled) {
			graphics.drawString(minecraft.font, Component.translatable("kairos.voice.muted"), MARGIN, y, Theme.MUTED, true);
			return;
		}
		if (VoiceClient.isTransmitting()) {
			graphics.drawString(minecraft.font, Component.literal("● ").append(Component.translatable("kairos.voice.talking")), MARGIN, y, Theme.ACCENT, true);
			y += LINE;
		}
		List<String> speaking = VoiceClient.speaking();
		for (String name : speaking) {
			graphics.drawString(minecraft.font, "♪ " + name, MARGIN, y, Theme.TEXT, true);
			y += LINE;
		}
	}
}
