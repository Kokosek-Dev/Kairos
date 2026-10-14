# Kairos in-game mod

The in-game side of the Kairos launcher, a Fabric mod for Minecraft 1.21.1.

- **Friends:** open the world you are playing to friends without a server. The mod asks your router (UPnP) to let them in and gives you a short code to send. A friend pastes the code in the same screen and joins. If the router cannot be asked, or your provider shares one address between many customers, only friends on the same network can join.
- **Skins:** shows the skins saved in the Kairos launcher and puts one on with a click, through the official Minecraft services. The launcher tells the game where its data lives (`-Dkairos.dir`), so this works when the game is started from Kairos.
- **Voice:** push to talk or open microphone, heard by players nearby and quieter the further away. It travels over the game's own connection, so a server needs the mod too (a world opened to friends has it already).

Open the menu with `K`, or with the Kairos button on the title and pause screens. Push to talk is `V`, and `N` switches voice chat on or off.

## Building

Needs JDK 25 to run the build (the mod itself targets Java 21).

```
set JAVA_HOME=<path to JDK 25>
gradlew build
```

The jar is in `build/libs`. Tests run with the build.

## What has been checked

Unit tests cover the join code, the UPnP parsing, the skin upload request, the voice codec and the server's limits. The mod has been started in the real game with Fabric API, and its screens were looked at. Voice between two players, opening a port on a real router and changing a skin on a real account have not been tried yet.