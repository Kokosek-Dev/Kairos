package dev.kairos.mod.skins;

import com.google.gson.Gson;
import com.google.gson.annotations.SerializedName;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;

/** The skins saved in the Kairos launcher. The launcher tells the game where its data lives. */
public final class SkinLibrary {
	/** The Java property the launcher sets to its data folder. */
	public static final String DIRECTORY_PROPERTY = "kairos.dir";
	private static final long MAX_INDEX_BYTES = 1024 * 1024;

	public record Skin(String id, String name, boolean slim) {
	}

	private record Index(List<Entry> skins) {
	}

	private record Entry(String id, String name, @SerializedName("variant") String variant) {
	}

	private SkinLibrary() {
	}

	public static Optional<Path> directory() {
		String value = System.getProperty(DIRECTORY_PROPERTY);
		return value == null || value.isBlank() ? Optional.empty() : Optional.of(Path.of(value, "skins"));
	}

	/** The saved skins, newest first as the launcher stores them; empty when the launcher did not start the game. */
	public static List<Skin> load() {
		Optional<Path> directory = directory();
		if (directory.isEmpty()) {
			return List.of();
		}
		try {
			Path file = directory.get().resolve("skins.json");
			if (Files.size(file) > MAX_INDEX_BYTES) {
				return List.of();
			}
			return parse(Files.readString(file));
		} catch (IOException | RuntimeException error) {
			return List.of();
		}
	}

	static List<Skin> parse(String json) {
		Index index = new Gson().fromJson(json, Index.class);
		if (index == null || index.skins() == null) {
			return List.of();
		}
		return index.skins().stream()
			.filter(entry -> entry.id() != null && entry.id().matches("[A-Za-z0-9_-]{1,64}") && entry.name() != null)
			.map(entry -> new Skin(entry.id(), entry.name(), "slim".equals(entry.variant())))
			.toList();
	}

	public static byte[] png(Skin skin) throws IOException {
		Path directory = directory().orElseThrow(() -> new IOException("the launcher did not share its folder"));
		return Files.readAllBytes(directory.resolve(skin.id() + ".png"));
	}
}
