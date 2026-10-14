package dev.kairos.mod.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.Button;
import net.minecraft.network.chat.Component;

/** A flat button in the Kairos colors; the main action of a screen can be drawn in the accent color. */
public final class FlatButton extends Button {
	private final boolean primary;

	public FlatButton(int x, int y, int width, int height, Component label, boolean primary, OnPress onPress) {
		super(x, y, width, height, label, onPress, DEFAULT_NARRATION);
		this.primary = primary;
	}

	@Override
	protected void renderWidget(GuiGraphics graphics, int mouseX, int mouseY, float delta) {
		boolean hover = isHoveredOrFocused() && active;
		int fill;
		if (!active) {
			fill = Theme.DISABLED;
		} else if (primary) {
			fill = hover ? Theme.ACCENT_HOVER : Theme.ACCENT;
		} else {
			fill = hover ? Theme.SURFACE_HOVER : Theme.SURFACE;
		}
		graphics.fill(getX(), getY(), getX() + width, getY() + height, fill);
		var font = Minecraft.getInstance().font;
		int color = !active ? Theme.MUTED : primary ? 0xFF1A0E02 : Theme.TEXT;
		graphics.drawCenteredString(font, getMessage(), getX() + width / 2, getY() + (height - 8) / 2, color);
	}
}
