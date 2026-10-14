package dev.kairos.mod.client.ui;

import com.mojang.blaze3d.platform.NativeImage;
import dev.kairos.mod.KairosMod;
import dev.kairos.mod.skins.SkinLibrary;
import dev.kairos.mod.skins.SkinUpload;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.ResourceLocation;

public final class SkinsScreen extends KairosScreen {
	private static final int ROWS = 4;
	private static final int ROW_HEIGHT = 30;

	private record Head(ResourceLocation texture, int height) {
	}

	private final List<SkinLibrary.Skin> skins = SkinLibrary.load();
	private final Map<String, Head> heads = new HashMap<>();
	private int page;
	private boolean busy;
	private Component status = Component.empty();
	private int statusColor = Theme.MUTED;
	private int contentX;
	private int contentY;
	private int contentWidth;

	public SkinsScreen(Screen parent) {
		super(parent, Tab.SKINS);
	}

	private int pages() {
		return Math.max(1, (skins.size() + ROWS - 1) / ROWS);
	}

	@Override
	protected void buildContent(int x, int y, int width, int height) {
		contentX = x;
		contentY = y;
		contentWidth = width;
		int first = page * ROWS;
		for (int row = 0; row < ROWS && first + row < skins.size(); row++) {
			SkinLibrary.Skin skin = skins.get(first + row);
			loadHead(skin);
			addRenderableWidget(new FlatButton(x + width - 64, y + row * ROW_HEIGHT + 3, 64, 20, Component.translatable("kairos.skins.wear"), true, pressed -> wear(skin)))
				.active = !busy;
		}
		if (pages() > 1) {
			addRenderableWidget(new FlatButton(x, y + ROWS * ROW_HEIGHT + 2, 20, 16, Component.literal("<"), false, pressed -> turn(-1))).active = page > 0;
			addRenderableWidget(new FlatButton(x + 60, y + ROWS * ROW_HEIGHT + 2, 20, 16, Component.literal(">"), false, pressed -> turn(1))).active = page < pages() - 1;
		}
	}

	private void turn(int direction) {
		page = Math.max(0, Math.min(pages() - 1, page + direction));
		rebuildWidgets();
	}

	private void loadHead(SkinLibrary.Skin skin) {
		if (heads.containsKey(skin.id())) {
			return;
		}
		try (NativeImage image = NativeImage.read(new ByteArrayInputStream(SkinLibrary.png(skin)))) {
			ResourceLocation id = ResourceLocation.fromNamespaceAndPath(KairosMod.ID, "skin_head/" + skin.id().toLowerCase());
			minecraft.getTextureManager().register(id, new DynamicTexture(copy(image)));
			heads.put(skin.id(), new Head(id, image.getHeight()));
		} catch (IOException | RuntimeException error) {
			KairosMod.LOGGER.warn("Could not read skin {}: {}", skin.id(), error.toString());
		}
	}

	private static NativeImage copy(NativeImage image) {
		NativeImage duplicate = new NativeImage(image.getWidth(), image.getHeight(), false);
		duplicate.copyFrom(image);
		return duplicate;
	}

	private void wear(SkinLibrary.Skin skin) {
		busy = true;
		status = Component.translatable("kairos.skins.working");
		statusColor = Theme.MUTED;
		rebuildWidgets();
		String token = minecraft.getUser().getAccessToken();
		Thread worker = new Thread(() -> {
			SkinUpload.Result result;
			try {
				result = SkinUpload.send(token, SkinLibrary.png(skin), skin.slim());
			} catch (IOException error) {
				result = SkinUpload.Result.FAILED;
			}
			SkinUpload.Result done = result;
			minecraft.execute(() -> finish(done));
		}, "Kairos-Skin");
		worker.setDaemon(true);
		worker.start();
	}

	private void finish(SkinUpload.Result result) {
		busy = false;
		status = Component.translatable("kairos.skins." + result.name().toLowerCase());
		statusColor = result == SkinUpload.Result.CHANGED ? Theme.SUCCESS : Theme.DANGER;
		if (minecraft.screen == this) {
			rebuildWidgets();
		}
	}

	@Override
	public void render(GuiGraphics graphics, int mouseX, int mouseY, float delta) {
		super.render(graphics, mouseX, mouseY, delta);
		if (SkinLibrary.directory().isEmpty()) {
			graphics.drawWordWrap(font, Component.translatable("kairos.skins.no_launcher"), contentX, contentY + 4, contentWidth, Theme.MUTED);
			return;
		}
		if (skins.isEmpty()) {
			graphics.drawWordWrap(font, Component.translatable("kairos.skins.empty"), contentX, contentY + 4, contentWidth, Theme.MUTED);
			return;
		}
		int first = page * ROWS;
		for (int row = 0; row < ROWS && first + row < skins.size(); row++) {
			SkinLibrary.Skin skin = skins.get(first + row);
			int rowTop = contentY + row * ROW_HEIGHT;
			graphics.fill(contentX, rowTop, contentX + contentWidth, rowTop + ROW_HEIGHT - 4, Theme.SURFACE);
			Head head = heads.get(skin.id());
			if (head != null) {
				graphics.blit(head.texture(), contentX + 4, rowTop + 3, 20, 20, 8f, 8f, 8, 8, 64, head.height());
				if (head.height() == 64) {
					graphics.blit(head.texture(), contentX + 4, rowTop + 3, 20, 20, 40f, 8f, 8, 8, 64, head.height());
				}
			}
			graphics.drawString(font, font.plainSubstrByWidth(skin.name(), contentWidth - 150), contentX + 32, rowTop + 4, Theme.TEXT, false);
			graphics.drawString(font, Component.translatable(skin.slim() ? "kairos.skins.slim" : "kairos.skins.classic"), contentX + 32, rowTop + 14, Theme.MUTED, false);
		}
		if (pages() > 1) {
			graphics.drawCenteredString(font, Component.translatable("kairos.skins.page", page + 1, pages()), contentX + 40, contentY + ROWS * ROW_HEIGHT + 6, Theme.MUTED);
		}
		graphics.drawString(font, status, contentX + 100, contentY + ROWS * ROW_HEIGHT + 6, statusColor, false);
	}

	@Override
	public void removed() {
		for (Head head : heads.values()) {
			minecraft.getTextureManager().release(head.texture());
		}
		heads.clear();
	}
}
