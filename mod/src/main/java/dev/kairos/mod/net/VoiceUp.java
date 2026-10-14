package dev.kairos.mod.net;

import dev.kairos.mod.KairosMod;
import dev.kairos.mod.voice.VoiceCodec;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;

/** One frame of a player's voice, on its way to the server. */
public record VoiceUp(int sequence, byte[] data) implements CustomPacketPayload {
	public static final Type<VoiceUp> TYPE = new Type<>(ResourceLocation.fromNamespaceAndPath(KairosMod.ID, "voice_up"));
	public static final StreamCodec<RegistryFriendlyByteBuf, VoiceUp> CODEC = CustomPacketPayload.codec(VoiceUp::write, VoiceUp::read);

	private void write(RegistryFriendlyByteBuf buffer) {
		buffer.writeVarInt(sequence);
		buffer.writeByteArray(data);
	}

	private static VoiceUp read(RegistryFriendlyByteBuf buffer) {
		return new VoiceUp(buffer.readVarInt(), buffer.readByteArray(VoiceCodec.MAX_PACKET));
	}

	@Override
	public Type<? extends CustomPacketPayload> type() {
		return TYPE;
	}
}
