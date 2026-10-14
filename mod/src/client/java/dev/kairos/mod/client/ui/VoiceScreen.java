package dev.kairos.mod.client.ui;

import dev.kairos.mod.client.Keys;
import dev.kairos.mod.client.voice.VoiceClient;
import dev.kairos.mod.client.voice.VoiceSettings;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

public final class VoiceScreen extends KairosScreen {
	private static final int ROW = 24;
	private final VoiceSettings settings = VoiceSettings.get();
	private int contentX;
	private int contentY;
	private int contentWidth;

	public VoiceScreen(Screen parent) {
		super(parent, Tab.VOICE);
	}

	@Override
	protected void buildContent(int x, int y, int width, int height) {
		contentX = x;
		contentY = y;
		contentWidth = width;
		int half = (width - 4) / 2;

		addRenderableWidget(new FlatButton(x, y, half, 20, onOff(), settings.enabled, pressed -> {
			settings.enabled = !settings.enabled;
			save();
		}));
		addRenderableWidget(new FlatButton(x + half + 4, y, half, 20, Component.translatable(settings.pushToTalk ? "kairos.voice.mode_push" : "kairos.voice.mode_open"), false, pressed -> {
			settings.pushToTalk = !settings.pushToTalk;
			save();
		}));
		addRenderableWidget(new FlatButton(x, y + ROW, width, 20, Component.translatable("kairos.voice.microphone", deviceLabel()), false, pressed -> {
			settings.inputDevice = nextDevice();
			save();
		}));
		addRenderableWidget(new FlatSlider(x, y + ROW * 2, half, 20, 0, 200, settings.volume,
			value -> Component.translatable("kairos.voice.volume", value), value -> {
				settings.volume = value;
				settings.save();
			}));
		if (!settings.pushToTalk) {
			addRenderableWidget(new FlatSlider(x + half + 4, y + ROW * 2, half, 20, 0, 100, settings.sensitivity,
				value -> Component.translatable("kairos.voice.sensitivity", value), value -> {
					settings.sensitivity = value;
					settings.save();
				}));
		}
	}

	private Component onOff() {
		return Component.translatable(settings.enabled ? "kairos.voice.on" : "kairos.voice.off");
	}

	private Component deviceLabel() {
		return settings.inputDevice.isEmpty() ? Component.translatable("kairos.voice.default_device") : Component.literal(settings.inputDevice);
	}

	private String nextDevice() {
		List<String> options = new ArrayList<>();
		options.add("");
		options.addAll(VoiceClient.inputDevices());
		int index = options.indexOf(settings.inputDevice);
		return options.get((index + 1) % options.size());
	}

	/** A change of microphone or switch takes effect the next time voice starts; saving also restarts it now. */
	private void save() {
		settings.save();
		if (VoiceClient.isRunning()) {
			VoiceClient.stop();
			Thread restart = new Thread(VoiceClient::start, "Kairos-Voice-Start");
			restart.setDaemon(true);
			restart.start();
		}
		rebuildWidgets();
	}

	@Override
	public void render(GuiGraphics graphics, int mouseX, int mouseY, float delta) {
		super.render(graphics, mouseX, mouseY, delta);
		int meterTop = contentY + ROW * 3 + 4;
		graphics.drawString(font, Component.translatable("kairos.voice.level"), contentX, meterTop, Theme.MUTED, false);
		int barLeft = contentX;
		int barTop = meterTop + 12;
		graphics.fill(barLeft, barTop, barLeft + contentWidth, barTop + 6, Theme.SURFACE);
		if (VoiceClient.isRunning()) {
			int filled = (int) (contentWidth * VoiceClient.level());
			graphics.fillGradient(barLeft, barTop, barLeft + filled, barTop + 6, VoiceClient.isTransmitting() ? Theme.ACCENT : Theme.MUTED, VoiceClient.isTransmitting() ? Theme.ACCENT_END : Theme.MUTED);
		}

		Component hint;
		int hintColor = Theme.MUTED;
		if (!VoiceClient.problem().isEmpty()) {
			hint = Component.translatable(VoiceClient.problem());
			hintColor = Theme.DANGER;
		} else if (!VoiceClient.isRunning()) {
			hint = Component.translatable("kairos.voice.not_running");
		} else if (settings.pushToTalk) {
			hint = Component.translatable("kairos.voice.hold", Keys.pushToTalk.getTranslatedKeyMessage());
		} else {
			hint = Component.translatable("kairos.voice.open_hint");
		}
		graphics.drawWordWrap(font, hint, contentX, barTop + 12, contentWidth, hintColor);
	}
}
