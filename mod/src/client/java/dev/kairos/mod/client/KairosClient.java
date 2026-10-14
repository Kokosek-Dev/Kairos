package dev.kairos.mod.client;

import dev.kairos.mod.client.friends.FriendsHost;
import dev.kairos.mod.client.ui.FlatButton;
import dev.kairos.mod.client.ui.KairosScreen;
import dev.kairos.mod.client.voice.VoiceClient;
import dev.kairos.mod.client.voice.VoiceHud;
import dev.kairos.mod.client.voice.VoiceSettings;
import dev.kairos.mod.net.VoiceDown;
import dev.kairos.mod.net.VoiceUp;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.fabricmc.fabric.api.client.screen.v1.ScreenEvents;
import net.fabricmc.fabric.api.client.screen.v1.Screens;
import net.minecraft.client.gui.screens.PauseScreen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.network.chat.Component;

public final class KairosClient implements ClientModInitializer {
	@Override
	public void onInitializeClient() {
		Keys.register();
		ClientTickEvents.END_CLIENT_TICK.register(minecraft -> {
			while (Keys.open.consumeClick()) {
				if (minecraft.screen == null) {
					minecraft.setScreen(KairosScreen.Tab.FRIENDS.open(null));
				}
			}
			while (Keys.mute.consumeClick()) {
				VoiceSettings settings = VoiceSettings.get();
				settings.enabled = !settings.enabled;
				settings.save();
			}
			VoiceClient.setPushed(Keys.pushToTalk.isDown() && minecraft.screen == null);
			VoiceClient.tick(minecraft);
		});

		ScreenEvents.AFTER_INIT.register((minecraft, screen, width, height) -> {
			if (screen instanceof TitleScreen) {
				Screens.getButtons(screen).add(new FlatButton(width - 74, 6, 68, 20, Component.literal("Kairos"), true,
					pressed -> minecraft.setScreen(KairosScreen.Tab.SKINS.open(screen))));
			} else if (screen instanceof PauseScreen) {
				Screens.getButtons(screen).add(new FlatButton(width - 74, 6, 68, 20, Component.literal("Kairos"), true,
					pressed -> minecraft.setScreen(KairosScreen.Tab.FRIENDS.open(screen))));
			}
		});

		ClientPlayNetworking.registerGlobalReceiver(VoiceDown.TYPE, (payload, context) -> VoiceClient.receive(payload));
		ClientPlayConnectionEvents.JOIN.register((handler, sender, minecraft) -> {
			if (ClientPlayNetworking.canSend(VoiceUp.TYPE)) {
				Thread start = new Thread(VoiceClient::start, "Kairos-Voice-Start");
				start.setDaemon(true);
				start.start();
			}
		});
		ClientPlayConnectionEvents.DISCONNECT.register((handler, minecraft) -> {
			FriendsHost.close();
			VoiceClient.stop();
		});
		HudRenderCallback.EVENT.register((graphics, tickCounter) -> VoiceHud.render(graphics));
	}
}
