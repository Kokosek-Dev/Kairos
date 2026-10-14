package dev.kairos.mod.friends;

import java.net.InetAddress;
import java.net.UnknownHostException;
import java.util.Locale;

/**
 * A short code that stands for an address and a port, so a friend can join by pasting it.
 * The code is the address and port with a one byte check, written in base 32 and split in groups of four.
 */
public final class JoinCode {
	private static final String ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
	private static final int GROUP = 4;

	public record Target(InetAddress address, int port) {
	}

	private JoinCode() {
	}

	public static String encode(InetAddress address, int port) {
		if (port < 1 || port > 65535) {
			throw new IllegalArgumentException("port out of range");
		}
		byte[] host = address.getAddress();
		byte[] bytes = new byte[host.length + 4];
		bytes[0] = (byte) host.length;
		System.arraycopy(host, 0, bytes, 1, host.length);
		bytes[host.length + 1] = (byte) (port >> 8);
		bytes[host.length + 2] = (byte) port;
		bytes[host.length + 3] = checksum(bytes, host.length + 3);
		String text = base32(bytes);
		StringBuilder grouped = new StringBuilder();
		for (int index = 0; index < text.length(); index += GROUP) {
			if (index > 0) {
				grouped.append('-');
			}
			grouped.append(text, index, Math.min(text.length(), index + GROUP));
		}
		return grouped.toString();
	}

	public static Target decode(String code) {
		String text = code.toUpperCase(Locale.ROOT).replaceAll("[\\s-]", "");
		byte[] bytes = fromBase32(text);
		if (bytes.length < 5) {
			throw new IllegalArgumentException("code is too short");
		}
		int length = bytes[0];
		if ((length != 4 && length != 16) || bytes.length != length + 4) {
			throw new IllegalArgumentException("code has an unexpected size");
		}
		if (bytes[bytes.length - 1] != checksum(bytes, bytes.length - 1)) {
			throw new IllegalArgumentException("code has a typo");
		}
		int port = ((bytes[length + 1] & 0xff) << 8) | (bytes[length + 2] & 0xff);
		if (port == 0) {
			throw new IllegalArgumentException("code has no port");
		}
		byte[] host = new byte[length];
		System.arraycopy(bytes, 1, host, 0, length);
		try {
			return new Target(InetAddress.getByAddress(host), port);
		} catch (UnknownHostException error) {
			throw new IllegalArgumentException("code has an invalid address", error);
		}
	}

	/** CRC-8, polynomial 0x07: catches any single mistyped character. */
	private static byte checksum(byte[] bytes, int count) {
		int crc = 0;
		for (int index = 0; index < count; index++) {
			crc ^= bytes[index] & 0xff;
			for (int bit = 0; bit < 8; bit++) {
				crc = (crc & 0x80) != 0 ? ((crc << 1) ^ 0x07) & 0xff : (crc << 1) & 0xff;
			}
		}
		return (byte) crc;
	}

	private static String base32(byte[] bytes) {
		StringBuilder text = new StringBuilder();
		int buffer = 0;
		int bits = 0;
		for (byte value : bytes) {
			buffer = (buffer << 8) | (value & 0xff);
			bits += 8;
			while (bits >= 5) {
				text.append(ALPHABET.charAt((buffer >> (bits - 5)) & 31));
				bits -= 5;
			}
		}
		if (bits > 0) {
			text.append(ALPHABET.charAt((buffer << (5 - bits)) & 31));
		}
		return text.toString();
	}

	private static byte[] fromBase32(String text) {
		int count = text.length() * 5 / 8;
		byte[] bytes = new byte[count];
		int buffer = 0;
		int bits = 0;
		int next = 0;
		for (int index = 0; index < text.length(); index++) {
			int value = ALPHABET.indexOf(text.charAt(index));
			if (value < 0) {
				throw new IllegalArgumentException("code has an unknown character");
			}
			buffer = (buffer << 5) | value;
			bits += 5;
			if (bits >= 8) {
				if (next < count) {
					bytes[next++] = (byte) (buffer >> (bits - 8));
				}
				bits -= 8;
			}
		}
		return bytes;
	}
}
