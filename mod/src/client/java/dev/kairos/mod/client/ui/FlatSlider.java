package dev.kairos.mod.client.ui;

import java.util.function.Consumer;
import java.util.function.IntFunction;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.network.chat.Component;

/** A slider over whole numbers in the Kairos colors. */
public final class FlatSlider extends AbstractSliderButton {
	private final int min;
	private final int max;
	private final IntFunction<Component> label;
	private final Consumer<Integer> onChange;

	public FlatSlider(int x, int y, int width, int height, int min, int max, int value, IntFunction<Component> label, Consumer<Integer> onChange) {
		super(x, y, width, height, label.apply(value), (double) (value - min) / (max - min));
		this.min = min;
		this.max = max;
		this.label = label;
		this.onChange = onChange;
	}

	private int current() {
		return min + (int) Math.round(value * (max - min));
	}

	@Override
	protected void updateMessage() {
		setMessage(label.apply(current()));
	}

	@Override
	protected void applyValue() {
		onChange.accept(current());
	}

	@Override
	public void renderWidget(GuiGraphics graphics, int mouseX, int mouseY, float delta) {
		graphics.fill(getX(), getY(), getX() + width, getY() + height, isHoveredOrFocused() ? Theme.SURFACE_HOVER : Theme.SURFACE);
		int filled = (int) (width * value);
		graphics.fillGradient(getX(), getY() + height - 3, getX() + filled, getY() + height, Theme.ACCENT, Theme.ACCENT_END);
		graphics.drawCenteredString(Minecraft.getInstance().font, getMessage(), getX() + width / 2, getY() + (height - 11) / 2, Theme.TEXT);
	}
}
