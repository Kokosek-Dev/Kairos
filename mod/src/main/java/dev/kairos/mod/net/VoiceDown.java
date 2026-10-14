package dev.kairos.mod.net;

import dev.kairos.mod.KairosMod;
import dev.kairos.mod.voice.VoiceCodec;
import java.util.UUID;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;

/** One frame of someone's voice, passed on by the server to a player close enough to hear it. */
public record VoiceDown(UUID speaker, int sequence, byte[] data) implements CustomPacketPayload {
	public static final Type<VoiceDown> TYPE = new Type<>(ResourceLocation.fromNamespaceAndPath(KairosMod.ID, "voice_down"));
	public static final StreamCodec<RegistryFriendlyByteBuf, VoiceDown> CODEC = CustomPacketPayload.codec(VoiceDown::write, VoiceDown::read);

	private void write(RegistryFriendlyByteBuf buffer) {
		buffer.writeUUID(speaker);
		buffer.writeVarInt(sequence);
		buffer.writeByteArray(data);
	}

	private static VoiceDown read(RegistryFriendlyByteBuf buffer) {
		return new VoiceDown(buffer.readUUID(), buffer.readVarInt(), buffer.readByteArray(VoiceCodec.MAX_PACKET));
	}

	@Override
	public Type<? extends CustomPacketPayload> type() {
		return TYPE;
	}
}
