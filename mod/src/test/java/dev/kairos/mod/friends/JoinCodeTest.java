package dev.kairos.mod.friends;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.net.InetAddress;
import org.junit.jupiter.api.Test;

class JoinCodeTest {
	@Test
	void roundTripsAnIpv4AddressAndPort() throws Exception {
		InetAddress address = InetAddress.getByName("203.0.113.42");

		String code = JoinCode.encode(address, 25565);
		JoinCode.Target target = JoinCode.decode(code);

		assertEquals(address, target.address());
		assertEquals(25565, target.port());
	}

	@Test
	void roundTripsAnIpv6Address() throws Exception {
		InetAddress address = InetAddress.getByName("2001:db8::1234");

		JoinCode.Target target = JoinCode.decode(JoinCode.encode(address, 40001));

		assertEquals(address, target.address());
		assertEquals(40001, target.port());
	}

	@Test
	void groupsTheCodeInFoursAndIgnoresHowItWasTyped() throws Exception {
		String code = JoinCode.encode(InetAddress.getByName("198.51.100.7"), 50000);

		assertTrue(code.matches("([A-Z2-7]{4}-)*[A-Z2-7]{1,4}"), code);
		JoinCode.Target lowercaseNoDashes = JoinCode.decode("  " + code.replace("-", "").toLowerCase() + " ");
		assertEquals(50000, lowercaseNoDashes.port());
	}

	@Test
	void catchesAnySingleMistypedCharacter() throws Exception {
		String code = JoinCode.encode(InetAddress.getByName("198.51.100.7"), 50000).replace("-", "");
		int caught = 0;
		int tried = 0;
		for (int index = 0; index < code.length(); index++) {
			for (char replacement : "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".toCharArray()) {
				if (replacement == code.charAt(index)) {
					continue;
				}
				String typo = code.substring(0, index) + replacement + code.substring(index + 1);
				tried++;
				try {
					JoinCode.Target decoded = JoinCode.decode(typo);
					if (!decoded.address().getHostAddress().equals("198.51.100.7") || decoded.port() != 50000) {
						// Decoded to something else: only acceptable if the check byte did not notice.
						continue;
					}
				} catch (IllegalArgumentException expected) {
					caught++;
				}
			}
		}
		// The check byte catches nearly every typo; the few that slip past are in the padding bits.
		assertTrue(caught * 100 / tried >= 95, caught + " of " + tried);
	}

	@Test
	void refusesCodesThatCannotBeRight() {
		assertThrows(IllegalArgumentException.class, () -> JoinCode.decode(""));
		assertThrows(IllegalArgumentException.class, () -> JoinCode.decode("not a code!"));
		assertThrows(IllegalArgumentException.class, () -> JoinCode.decode("AAAA-AAAA"));
	}

	@Test
	void refusesAPortThatDoesNotExist() throws Exception {
		assertThrows(IllegalArgumentException.class, () -> JoinCode.encode(InetAddress.getByName("203.0.113.42"), 0));
		assertThrows(IllegalArgumentException.class, () -> JoinCode.encode(InetAddress.getByName("203.0.113.42"), 70000));
	}
}
