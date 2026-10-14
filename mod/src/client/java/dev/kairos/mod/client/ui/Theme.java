package dev.kairos.mod.client.ui;

import net.minecraft.client.gui.GuiGraphics;

/** The look of Kairos, matching the launcher: dark panels with an orange accent. */
public final class Theme {
	public static final int DIM = 0xB0060810;
	public static final int PANEL = 0xF0101424;
	public static final int BORDER = 0xFF2A3050;
	public static final int SURFACE = 0xFF1B2138;
	public static final int SURFACE_HOVER = 0xFF262E4C;
	public static final int DISABLED = 0xFF141829;
	public static final int TEXT = 0xFFE8ECF8;
	public static final int MUTED = 0xFF8A93B0;
	public static final int ACCENT = 0xFFFF7A18;
	public static final int ACCENT_HOVER = 0xFFFF9340;
	public static final int ACCENT_END = 0xFFFF5E7E;
	public static final int SUCCESS = 0xFF4ADE80;
	public static final int DANGER = 0xFFF87171;

	private Theme() {
	}

	public static void panel(GuiGraphics graphics, int x, int y, int width, int height) {
		graphics.fill(x - 1, y - 1, x + width + 1, y + height + 1, BORDER);
		graphics.fill(x, y, x + width, y + height, PANEL);
	}

	/** The thin orange-to-pink line under a header. */
	public static void accentLine(GuiGraphics graphics, int x, int y, int width) {
		graphics.fillGradient(x, y, x + width, y + 2, ACCENT, ACCENT_END);
	}
}
