package dev.kairos.mod.skins;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.UUID;

/** Changes the player's skin through the official Minecraft services, using the session of the running game. */
public final class SkinUpload {
	public enum Result {
		CHANGED, SIGNED_OUT, REFUSED, TOO_MANY, FAILED
	}

	private static final String ENDPOINT = "https://api.minecraftservices.com/minecraft/profile/skins";

	private SkinUpload() {
	}

	public static Result send(String accessToken, byte[] png, boolean slim) {
		return send(ENDPOINT, accessToken, png, slim);
	}

	static Result send(String endpoint, String accessToken, byte[] png, boolean slim) {
		String boundary = "kairos-" + UUID.randomUUID();
		try {
			HttpRequest request = HttpRequest.newBuilder(URI.create(endpoint))
				.timeout(Duration.ofSeconds(20))
				.header("Authorization", "Bearer " + accessToken)
				.header("Content-Type", "multipart/form-data; boundary=" + boundary)
				.POST(HttpRequest.BodyPublishers.ofByteArray(body(boundary, png, slim)))
				.build();
			HttpResponse<Void> response = HttpClient.newHttpClient().send(request, HttpResponse.BodyHandlers.discarding());
			return interpret(response.statusCode());
		} catch (InterruptedException error) {
			Thread.currentThread().interrupt();
			return Result.FAILED;
		} catch (IOException | IllegalArgumentException error) {
			return Result.FAILED;
		}
	}

	static Result interpret(int status) {
		if (status >= 200 && status < 300) {
			return Result.CHANGED;
		}
		return switch (status) {
			case 401, 403 -> Result.SIGNED_OUT;
			case 400 -> Result.REFUSED;
			case 429 -> Result.TOO_MANY;
			default -> Result.FAILED;
		};
	}

	static byte[] body(String boundary, byte[] png, boolean slim) throws IOException {
		ByteArrayOutputStream out = new ByteArrayOutputStream();
		out.write(("--" + boundary + "\r\nContent-Disposition: form-data; name=\"variant\"\r\n\r\n" + (slim ? "slim" : "classic") + "\r\n").getBytes(StandardCharsets.UTF_8));
		out.write(("--" + boundary + "\r\nContent-Disposition: form-data; name=\"file\"; filename=\"skin.png\"\r\nContent-Type: image/png\r\n\r\n").getBytes(StandardCharsets.UTF_8));
		out.write(png);
		out.write(("\r\n--" + boundary + "--\r\n").getBytes(StandardCharsets.UTF_8));
		return out.toByteArray();
	}
}
