# Zenith

An independent launcher for Minecraft: Java Edition, built with Tauri 2 (Rust) and React.

- Microsoft sign-in in your browser (no codes to type), several accounts, tokens kept in the system credential store
- Skins: 3D preview, a skin library, copy a skin from any player, capes
- Instances with their own version, loader, memory and Java settings
- Fabric, Quilt, Forge and NeoForge
- Mod, modpack, resource pack and shader browser with dependency-aware installs and updates, `.mrpack` import and export
- Automatic Java download, live game log, English and Polish

Zenith is not affiliated with Mojang or Microsoft, see [NOTICE.md](NOTICE.md). MIT licensed.

## Building

Requirements: Node.js, Rust (MSVC toolchain on Windows) and the Visual Studio C++ build tools.

```
npm install
npm run tauri dev       # development
npm run tauri build     # installer (NSIS) in src-tauri/target/release/bundle
```

### Microsoft sign-in

Sign-in needs an Azure application that Microsoft approved for Minecraft services. Register it once, put its
Application (client) ID into `src-tauri/.cargo/config.toml` (`ZENITH_MS_CLIENT_ID`) and build. Players never see
or enter this ID. Step by step: [docs/AZURE_SETUP.md](docs/AZURE_SETUP.md).

For development without a Microsoft account, debug builds start with a placeholder identity when
`ZENITH_DEV_SESSION=1` is set. That identity cannot join online servers and does not exist in release builds.
