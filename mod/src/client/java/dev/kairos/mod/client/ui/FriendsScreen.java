package dev.kairos.mod.client.ui;

import dev.kairos.mod.client.friends.FriendsHost;
import dev.kairos.mod.friends.JoinCode;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.screens.ConnectScreen;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.multiplayer.ServerData;
import net.minecraft.client.multiplayer.resolver.ServerAddress;
import net.minecraft.network.chat.Component;

public final class FriendsScreen extends KairosScreen {
	private EditBox codeBox;
	private Component joinStatus = Component.empty();
	private String shown = "";
	private int contentX;
	private int contentY;

	public FriendsScreen(Screen parent) {
		super(parent, Tab.FRIENDS);
	}

	/** What the host part of the screen depends on, so it can be rebuilt when the router answers. */
	private static String hostSnapshot() {
		return FriendsHost.state() + "|" + FriendsHost.note() + "|" + FriendsHost.internetCode() + "|" + FriendsHost.networkCode();
	}

	@Override
	protected void buildContent(int x, int y, int width, int height) {
		contentX = x;
		contentY = y;
		shown = hostSnapshot();

		if (FriendsHost.canOpen(minecraft)) {
			addRenderableWidget(new FlatButton(x, y + 12, 150, 20, Component.translatable("kairos.friends.open"), true, pressed -> {
				FriendsHost.open(minecraft);
				rebuildWidgets();
			}));
		}
		if (FriendsHost.state() != FriendsHost.State.IDLE) {
			addRenderableWidget(new FlatButton(x + width - 90, y + 12, 90, 20, Component.translatable("kairos.friends.close"), false, pressed -> {
				FriendsHost.close();
				rebuildWidgets();
			}));
		}
		if (FriendsHost.internetCode() != null) {
			addCopyButton(x + width - 50, y + 47, FriendsHost.internetCode());
		}
		if (FriendsHost.networkCode() != null) {
			addCopyButton(x + width - 50, y + 69, FriendsHost.networkCode());
		}

		codeBox = new EditBox(font, x, y + 112, width - 70, 20, Component.translatable("kairos.friends.code"));
		codeBox.setMaxLength(60);
		codeBox.setHint(Component.translatable("kairos.friends.code_hint"));
		addRenderableWidget(codeBox);
		addRenderableWidget(new FlatButton(x + width - 64, y + 112, 64, 20, Component.translatable("kairos.friends.join"), true, pressed -> join()));
	}

	private void addCopyButton(int x, int y, String code) {
		addRenderableWidget(new FlatButton(x, y, 50, 14, Component.translatable("kairos.friends.copy"), false, pressed -> minecraft.keyboardHandler.setClipboard(code)));
	}

	private void join() {
		try {
			JoinCode.Target target = JoinCode.decode(codeBox.getValue());
			String host = target.address().getHostAddress();
			ServerAddress address = new ServerAddress(host, target.port());
			ServerData data = new ServerData("Kairos", host + ":" + target.port(), ServerData.Type.OTHER);
			ConnectScreen.startConnecting(this, minecraft, address, data, false, null);
		} catch (IllegalArgumentException error) {
			joinStatus = Component.translatable("kairos.friends.bad_code");
		}
	}

	@Override
	public void tick() {
		if (!shown.equals(hostSnapshot())) {
			rebuildWidgets();
		}
	}

	@Override
	public void render(GuiGraphics graphics, int mouseX, int mouseY, float delta) {
		super.render(graphics, mouseX, mouseY, delta);
		graphics.drawString(font, Component.translatable("kairos.friends.host"), contentX, contentY, Theme.TEXT, false);

		String note = FriendsHost.note();
		if (note.isEmpty() && !FriendsHost.canOpen(minecraft)) {
			note = "kairos.friends.need_world";
		}
		if (!note.isEmpty()) {
			int color = note.endsWith("ready") ? Theme.SUCCESS : Theme.MUTED;
			graphics.drawString(font, Component.translatable(note), contentX, contentY + 38, color, false);
		}
		drawCode(graphics, "kairos.friends.code_internet", FriendsHost.internetCode(), contentY + 50);
		drawCode(graphics, "kairos.friends.code_network", FriendsHost.networkCode(), contentY + 72);

		graphics.drawString(font, Component.translatable("kairos.friends.join_title"), contentX, contentY + 98, Theme.TEXT, false);
		graphics.drawString(font, joinStatus, contentX, contentY + 136, Theme.DANGER, false);
	}

	private void drawCode(GuiGraphics graphics, String label, String code, int line) {
		if (code == null) {
			return;
		}
		graphics.drawString(font, Component.translatable(label), contentX, line, Theme.MUTED, false);
		graphics.drawString(font, code, contentX + font.width(Component.translatable(label)) + 8, line, Theme.ACCENT, false);
	}
}
