package dev.kairos.mod.skins;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import org.junit.jupiter.api.Test;

class SkinsTest {
	@Test
	void readsTheLaunchersSkinIndex() {
		String json = "{\"skins\":[{\"id\":\"ab12\",\"name\":\"Steve\",\"variant\":\"classic\",\"addedAt\":\"2026-10-10T10:00:00Z\"},"
			+ "{\"id\":\"cd34\",\"name\":\"Alex\",\"variant\":\"slim\",\"addedAt\":\"2026-10-11T10:00:00Z\"}]}";

		List<SkinLibrary.Skin> skins = SkinLibrary.parse(json);

		assertEquals(List.of(new SkinLibrary.Skin("ab12", "Steve", false), new SkinLibrary.Skin("cd34", "Alex", true)), skins);
	}

	@Test
	void skipsEntriesWhoseIdCouldPointOutsideTheFolder() {
		String json = "{\"skins\":[{\"id\":\"../../secret\",\"name\":\"Bad\",\"variant\":\"classic\"},{\"id\":\"fine\",\"name\":\"Good\",\"variant\":\"classic\"}]}";

		assertEquals(List.of(new SkinLibrary.Skin("fine", "Good", false)), SkinLibrary.parse(json));
	}

	@Test
	void treatsAnEmptyOrBrokenIndexAsNoSkins() {
		assertEquals(List.of(), SkinLibrary.parse("{}"));
		assertEquals(List.of(), SkinLibrary.parse(""));
	}

	@Test
	void buildsAMultipartBodyWithTheVariantAndTheFile() throws Exception {
		byte[] png = {1, 2, 3, 4};

		String body = new String(SkinUpload.body("XYZ", png, true), StandardCharsets.ISO_8859_1);

		assertTrue(body.startsWith("--XYZ\r\nContent-Disposition: form-data; name=\"variant\"\r\n\r\nslim\r\n"));
		assertTrue(body.contains("name=\"file\"; filename=\"skin.png\"\r\nContent-Type: image/png\r\n\r\n\u0001\u0002\u0003\u0004\r\n--XYZ--\r\n"));
	}

	@Test
	void explainsWhatTheServiceAnswered() {
		assertEquals(SkinUpload.Result.CHANGED, SkinUpload.interpret(200));
		assertEquals(SkinUpload.Result.SIGNED_OUT, SkinUpload.interpret(401));
		assertEquals(SkinUpload.Result.REFUSED, SkinUpload.interpret(400));
		assertEquals(SkinUpload.Result.TOO_MANY, SkinUpload.interpret(429));
		assertEquals(SkinUpload.Result.FAILED, SkinUpload.interpret(500));
	}
}
