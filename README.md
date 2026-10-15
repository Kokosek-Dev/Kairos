# Kairos

An independent launcher for Minecraft, built with Tauri 2 (Rust) and React. Java Edition instances with mods and loaders, and Bedrock Edition instances for add-on packs.

💬 **Community:** [Discord server](https://discord.gg/H3Mecyf3H9) · [Website](https://kokosek-dev.github.io/Kairos/) · made by [Kokosek-Dev](https://github.com/Kokosek-Dev)

## Download

Get the latest installer from the **Releases** page of this repository (`Kairos_x.y.z_x64-setup.exe`). The installer
installs for your user only and does not need administrator rights. Kairos keeps itself up to date.

### Check what you downloaded

Every release has a `SHA256SUMS.txt`. Compare it with your file in PowerShell:

```
Get-FileHash .\Kairos_0.1.0_x64-setup.exe -Algorithm SHA256
```

### Windows says "unknown publisher"

The installer is not signed with a paid code signing certificate yet, so Windows SmartScreen may warn you. Choose
**More info**, then **Run anyway**. The code is open and every release is built automatically from it, so you can
verify it yourself. See [docs/PRIVACY.md](docs/PRIVACY.md) for what Kairos does on your computer and on the network.

## Features

- Microsoft sign-in in your browser (no codes to type), several accounts, tokens kept in the system credential store
- Home page with your skin in 3D, a skin library and capes
- Skin editor: paint on the flat texture or directly on the 3D model
- Texture pack creator (beta): redraw any texture of a game version in a full-screen editor with a 3D preview of blocks, items and mobs as the game draws them, shapes, mirror drawing and any resolution; Kairos switches the pack on in the instances you pick
- Bedrock Edition instances next to the Java ones: each keeps its own add-on packs (`.mcpack`, `.mcaddon`) and Play puts them into the game
- Instances with their own version, loader, memory and Java settings, with memory suggested for your computer
- Fabric, Quilt, Forge and NeoForge
- Mod, modpack, resource pack and shader browser with dependency-aware installs and updates, automatic mod updates
  before a game starts, `.mrpack` import and export
- Boost: one click installs performance mods for the loader and tunes memory and Java settings
- Kairos codes: share an instance as a short text code that a friend pastes to build the same game
- Discord Rich Presence (optional, see [docs/DISCORD_SETUP.md](docs/DISCORD_SETUP.md))
- Moments: snapshots of an instance (mods, settings, optionally worlds) saved before risky changes, with a one-click way back and a view of what changed
- Mod detective: finds which mod breaks the game by switching mods off in halves
- Journal: a calendar of the days you played, with streaks and time per instance
- Server list for Java and Bedrock servers with live status (players, ping, description) and a one-click Join for Bedrock
- Play time statistics, crash analysis that explains what went wrong, a panel to send feedback or report a problem (you choose what is sent)
- Kairos mod for the game (Fabric 1.21.1): switch skins, voice chat for nearby players and play with friends without a server, see [mod/README.md](mod/README.md)
- Choose where everything is stored, automatic Java download, live game log
- English, Polish, German, Spanish, French, Brazilian Portuguese and Russian, dark and light themes, accent colors and backgrounds

You need a Microsoft account that owns Minecraft to play. Bedrock Edition is installed and run by Microsoft's own app. Kairos does not bypass the game's licence.

Kairos is not affiliated with Mojang or Microsoft, see [NOTICE.md](NOTICE.md). MIT licensed.

Developers: build and release notes are in [docs/](docs/), and the in-game mod has its own [README](mod/README.md).
