package dev.kairos.mod.client.ui;

import dev.kairos.mod.KairosMod;
import java.util.function.Function;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.ResourceLocation;

/** The frame every Kairos screen shares: the panel, the header with the logo, and the tabs. */
public abstract class KairosScreen extends Screen {
	public enum Tab {
		SKINS("kairos.tab.skins", SkinsScreen::new),
		FRIENDS("kairos.tab.friends", FriendsScreen::new),
		VOICE("kairos.tab.voice", VoiceScreen::new);

		private final String key;
		private final Function<Screen, KairosScreen> create;

		Tab(String key, Function<Screen, KairosScreen> create) {
			this.key = key;
			this.create = create;
		}

		public KairosScreen open(Screen parent) {
			return create.apply(parent);
		}
	}

	private static final ResourceLocation LOGO = ResourceLocation.fromNamespaceAndPath(KairosMod.ID, "textures/gui/logo.png");
	protected static final int WIDTH = 360;
	protected static final int HEIGHT = 230;
	private static final int PADDING = 14;
	private static final int HEADER = 30;

	protected final Screen parent;
	private final Tab tab;
	protected int left;
	protected int top;

	protected KairosScreen(Screen parent, Tab tab) {
		super(Component.translatable(tab.key));
		this.parent = parent;
		this.tab = tab;
	}

	@Override
	protected void init() {
		left = (width - WIDTH) / 2;
		top = (height - HEIGHT) / 2;
		Tab[] tabs = Tab.values();
		int gap = 4;
		int tabWidth = (WIDTH - PADDING * 2 - gap * (tabs.length - 1)) / tabs.length;
		for (int index = 0; index < tabs.length; index++) {
			Tab next = tabs[index];
			var button = new FlatButton(left + PADDING + index * (tabWidth + gap), top + HEADER + 8, tabWidth, 20,
				Component.translatable(next.key), next == tab, pressed -> minecraft.setScreen(next.open(parent)));
			button.active = next != tab;
			addRenderableWidget(button);
		}
		addRenderableWidget(new FlatButton(left + WIDTH - PADDING - 20, top + 5, 20, 20, Component.literal("x"), false, pressed -> onClose()));
		int contentTop = top + HEADER + 38;
		buildContent(left + PADDING, contentTop, WIDTH - PADDING * 2, top + HEIGHT - PADDING - contentTop);
	}

	/** Adds this tab's own widgets inside the given area. */
	protected abstract void buildContent(int x, int y, int width, int height);

	@Override
	public void renderBackground(GuiGraphics graphics, int mouseX, int mouseY, float delta) {
		graphics.fill(0, 0, width, height, Theme.DIM);
		Theme.panel(graphics, left, top, WIDTH, HEIGHT);
		graphics.blit(LOGO, left + PADDING, top + 6, 18, 18, 0f, 0f, 128, 128, 128, 128);
		graphics.drawString(font, "KAIROS", left + PADDING + 24, top + 11, Theme.TEXT, false);
		Theme.accentLine(graphics, left, top + HEADER, WIDTH);
	}

	@Override
	public boolean isPauseScreen() {
		return false;
	}

	@Override
	public void onClose() {
		minecraft.setScreen(parent);
	}
}
