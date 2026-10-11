# Privacy

Zenith has no accounts of its own, no telemetry, no analytics and no advertising. Nothing about how you use it is
sent to the people who make Zenith.

## What stays on your computer

- **Your files.** Games, mods, skins, worlds and settings live in the folder shown under Settings, Storage. You can
  move it to another drive at any time.
- **Your Microsoft sign-in.** Only the long-lived sign-in token is kept, in the Windows Credential Manager. Short-lived
  tokens exist only in memory while the launcher runs. Zenith never sees your Microsoft password: you type it on
  Microsoft's own page in your browser.
- **Your preferences.** Language, color, background and similar choices are stored locally.

## Who Zenith talks to

Zenith connects to these services, and only to do what its screens say:

| Service | Used for |
| --- | --- |
| `login.microsoftonline.com`, `user.auth.xboxlive.com`, `xsts.auth.xboxlive.com`, `api.minecraftservices.com` | Signing you in and reading your Minecraft profile, skin and capes |
| `launchermeta.mojang.com`, `piston-meta.mojang.com`, `piston-data.mojang.com`, `libraries.minecraft.net`, `resources.download.minecraft.net` | Game versions, libraries and assets |
| `api.mojang.com`, `sessionserver.mojang.com`, `textures.minecraft.net` | Looking up another player's skin when you ask for it |
| `api.adoptium.net` | Downloading the Java runtime a game needs |
| `meta.fabricmc.net`, `meta.quiltmc.org`, `maven.fabricmc.net`, `maven.minecraftforge.net`, `files.minecraftforge.net`, `maven.neoforged.net` | Installing mod loaders |
| `api.modrinth.com`, `cdn.modrinth.com` | Searching for mods, modpacks, resource packs and shaders, and downloading them |
| `github.com` | Checking for a newer Zenith, and downloading files that a modpack points to |

Like any web request, these reveal your IP address to the service you contact. Zenith identifies itself to the mod
catalogue as `zenith-launcher/zenith/<version>` and sends no personal details.

## Problem reports

Settings, Help can save a problem report as a zip file on your computer. It holds logs, your instance and mod names
and basic system information. Values that sign you in and your Windows user name are removed from it. Zenith never
sends the report anywhere: you choose whether and with whom to share it.

## Questions

Zenith is open source. Everything above can be checked in the code.
