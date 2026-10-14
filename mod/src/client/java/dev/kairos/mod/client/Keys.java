package dev.kairos.mod.client;

import com.mojang.blaze3d.platform.InputConstants;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.KeyMapping;
import org.lwjgl.glfw.GLFW;

/** The keys Kairos adds to the game's controls. They have to be registered while the mod starts, before the game reads its options. */
public final class Keys {
	private static final String CATEGORY = "key.categories.kairos";

	public static KeyMapping open;
	public static KeyMapping pushToTalk;
	public static KeyMapping mute;

	private Keys() {
	}

	public static void register() {
		open = register("key.kairos.open", GLFW.GLFW_KEY_K);
		pushToTalk = register("key.kairos.push_to_talk", GLFW.GLFW_KEY_V);
		mute = register("key.kairos.mute", GLFW.GLFW_KEY_N);
	}

	private static KeyMapping register(String name, int key) {
		return KeyBindingHelper.registerKeyBinding(new KeyMapping(name, InputConstants.Type.KEYSYM, key, CATEGORY));
	}
}
