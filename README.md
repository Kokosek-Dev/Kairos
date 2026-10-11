# Zenith

An independent launcher for Minecraft: Java Edition, built with Tauri 2 (Rust) and React.

## Download

Get the latest installer from the **Releases** page of this repository (`Zenith_x.y.z_x64-setup.exe`). The installer
installs for your user only and does not need administrator rights. Zenith keeps itself up to date.

### Check what you downloaded

Every release has a `SHA256SUMS.txt`. Compare it with your file in PowerShell:

```
Get-FileHash .\Zenith_0.1.0_x64-setup.exe -Algorithm SHA256
```

### Windows says "unknown publisher"

The installer is not signed with a paid code signing certificate yet, so Windows SmartScreen may warn you. Choose
**More info**, then **Run anyway**. The code is open and every release is built automatically from it, so you can
verify it yourself. See [docs/PRIVACY.md](docs/PRIVACY.md) for what Zenith does on your computer and on the network.

## Features

- Microsoft sign-in in your browser (no codes to type), several accounts, tokens kept in the system credential store
- Home page with your skin in 3D, a skin library and capes
- Skin editor: paint on the flat texture or directly on the 3D model
- Texture pack creator (beta): redraw any texture of a game version, Zenith switches the pack on in the instances you pick
- Instances with their own version, loader, memory and Java settings, with memory suggested for your computer
- Fabric, Quilt, Forge and NeoForge
- Mod, modpack, resource pack and shader browser with dependency-aware installs and updates, automatic mod updates
  before a game starts, `.mrpack` import and export
- Play time statistics, crash analysis that explains what went wrong, problem reports without personal data
- Choose where everything is stored, automatic Java download, live game log
- English and Polish, accent colors and backgrounds

You need a Microsoft account that owns Minecraft: Java Edition to play. Zenith does not bypass the game's licence.

Zenith is not affiliated with Mojang or Microsoft, see [NOTICE.md](NOTICE.md). MIT licensed.

## Building

Requirements: Node.js, Rust (MSVC toolchain on Windows) and the Visual Studio C++ build tools.

```
npm install
npm run tauri dev       # development
npm run tauri build     # installer (NSIS) in src-tauri/target/release/bundle
```

`tauri build` signs the update package, so it needs the update signing key in `TAURI_SIGNING_PRIVATE_KEY`.
Releases are built by GitHub Actions, see [docs/RELEASING.md](docs/RELEASING.md).

### Microsoft sign-in

Sign-in needs an Azure application that Microsoft approved for Minecraft services. Register it once, put its
Application (client) ID into `src-tauri/.cargo/config.toml` (`ZENITH_MS_CLIENT_ID`) and build. Players never see
or enter this ID. Step by step: [docs/AZURE_SETUP.md](docs/AZURE_SETUP.md).

For development without a Microsoft account, debug builds start with a placeholder identity when
`ZENITH_DEV_SESSION=1` is set. That identity cannot join online servers and does not exist in release builds.
