use crate::error::{AppError, AppResult};
use crate::instances::model::Instance;
use crate::instances::store as instances;
use crate::mojang::profile::is_safe_id;
use crate::paths::DataPaths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

const TEXTURES_ROOT: &str = "assets/minecraft/textures";
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const MAX_TEXTURE_BYTES: usize = 4 * 1024 * 1024;
const MAX_TEXTURE_SIDE: u32 = 4096;
const MAX_NAME_CHARS: usize = 48;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub pack_format: u32,
    /// Newer versions describe the supported range with `min_format` and `max_format`.
    pub ranged_format: bool,
    /// Instances that use this pack; Kairos keeps their copy up to date and switched on.
    #[serde(default)]
    pub instances: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureList {
    pub textures: Vec<String>,
    pub edited: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Preferences {
    #[serde(default = "enabled")]
    packs_everywhere: bool,
}

fn enabled() -> bool {
    true
}

fn invalid(message: &str) -> AppError {
    AppError::InvalidInput(message.to_owned())
}

fn zip_error(error: zip::result::ZipError) -> AppError {
    AppError::Internal(error.to_string())
}

fn is_pack_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn pack_dir(paths: &DataPaths, id: &str) -> AppResult<PathBuf> {
    if !is_pack_id(id) {
        return Err(invalid("invalid texture pack id"));
    }
    Ok(paths.texture_packs().join(id))
}

/// Texture paths come from the frontend, so only plain lowercase resource names are accepted.
fn is_texture_path(path: &str) -> bool {
    path.len() <= 200
        && path.ends_with(".png")
        && path.split('/').all(|part| {
            !part.is_empty() && part != "." && part != ".." && part.bytes().all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.'))
        })
}

fn checked_texture_path(path: &str) -> AppResult<()> {
    is_texture_path(path).then_some(()).ok_or_else(|| invalid("invalid texture path"))
}

fn texture_file(dir: &Path, path: &str) -> PathBuf {
    path.split('/').fold(dir.join(TEXTURES_ROOT), |base, part| base.join(part))
}

fn open_jar(paths: &DataPaths, game_version: &str) -> AppResult<zip::ZipArchive<File>> {
    if !is_safe_id(game_version) {
        return Err(AppError::VersionNotFound(game_version.to_owned()));
    }
    let file = File::open(paths.client_jar(game_version)).map_err(|_| AppError::VersionNotFound(game_version.to_owned()))?;
    zip::ZipArchive::new(file).map_err(zip_error)
}

/// Reads the resource pack format the game version expects from the `version.json` inside its jar.
fn pack_format_of(jar: &mut zip::ZipArchive<File>) -> AppResult<(u32, bool)> {
    let mut entry = jar.by_name("version.json").map_err(|_| invalid("this version cannot be used for texture packs"))?;
    let mut text = String::new();
    entry.read_to_string(&mut text)?;
    let meta: serde_json::Value = serde_json::from_str(&text)?;
    let pack_version = &meta["pack_version"];
    let format = if let Some(number) = pack_version.as_u64() {
        Some((number, false))
    } else if let Some(major) = pack_version["resource_major"].as_u64() {
        Some((major, true))
    } else {
        pack_version["resource"].as_u64().map(|number| (number, false))
    };
    format
        .and_then(|(number, ranged)| u32::try_from(number).ok().map(|number| (number, ranged)))
        .ok_or_else(|| invalid("this version does not say which resource pack format it uses"))
}

/// Installed vanilla versions whose game jar is available to take textures from.
pub fn available_versions(paths: &DataPaths) -> AppResult<Vec<String>> {
    let Ok(entries) = fs::read_dir(paths.versions()) else {
        return Ok(Vec::new());
    };
    let mut versions: Vec<String> = entries
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|id| paths.client_jar(id).is_file() && paths.version_json(id).is_file())
        .filter(|id| open_jar(paths, id).and_then(|mut jar| pack_format_of(&mut jar)).is_ok())
        .collect();
    versions.sort();
    Ok(versions)
}

fn write_info(dir: &Path, info: &PackInfo) -> AppResult<()> {
    let target = dir.join("pack.json");
    let temp = dir.join("pack.json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(info)?)?;
    fs::rename(temp, target)?;
    Ok(())
}

pub fn create(paths: &DataPaths, name: &str, game_version: &str) -> AppResult<PackInfo> {
    let name: String = name.trim().chars().filter(|character| !character.is_control()).take(MAX_NAME_CHARS).collect();
    if name.is_empty() {
        return Err(invalid("give the texture pack a name"));
    }
    let (pack_format, ranged_format) = pack_format_of(&mut open_jar(paths, game_version)?)?;
    let mut id_bytes = [0u8; 8];
    getrandom::fill(&mut id_bytes).map_err(|error| AppError::Internal(error.to_string()))?;
    let info = PackInfo {
        id: hex::encode(id_bytes),
        name,
        game_version: game_version.to_owned(),
        pack_format,
        ranged_format,
        instances: Vec::new(),
        created_at: Utc::now(),
    };
    let dir = pack_dir(paths, &info.id)?;
    fs::create_dir_all(&dir)?;
    write_info(&dir, &info)?;
    place_where_used(paths, &info);
    Ok(info)
}

pub fn info(paths: &DataPaths, id: &str) -> AppResult<PackInfo> {
    let bytes = fs::read(pack_dir(paths, id)?.join("pack.json")).map_err(|_| invalid("texture pack not found"))?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn list(paths: &DataPaths) -> AppResult<Vec<PackInfo>> {
    let Ok(entries) = fs::read_dir(paths.texture_packs()) else {
        return Ok(Vec::new());
    };
    let mut packs: Vec<PackInfo> = entries
        .flatten()
        .filter_map(|entry| info(paths, &entry.file_name().to_string_lossy()).ok())
        .collect();
    packs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(packs)
}

pub fn delete(paths: &DataPaths, id: &str) -> AppResult<()> {
    let dir = pack_dir(paths, id)?;
    if let Ok(pack) = info(paths, id) {
        for instance in instances::list(paths).unwrap_or_default() {
            remove_from_instance(paths, &pack, &instance.id);
        }
    }
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

fn collect_edited(dir: &Path, prefix: &str, found: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let relative = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
        if entry.path().is_dir() {
            collect_edited(&entry.path(), &relative, found);
        } else if is_texture_path(&relative) {
            found.push(relative);
        }
    }
}

fn edited_textures(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    collect_edited(&dir.join(TEXTURES_ROOT), "", &mut found);
    found.sort();
    found
}

pub fn list_textures(paths: &DataPaths, id: &str) -> AppResult<TextureList> {
    let pack = info(paths, id)?;
    let jar = open_jar(paths, &pack.game_version)?;
    let prefix = format!("{TEXTURES_ROOT}/");
    let mut textures: Vec<String> = jar
        .file_names()
        .flatten()
        .filter_map(|name| name.strip_prefix(&prefix).map(str::to_owned))
        .filter(|name| is_texture_path(name))
        .collect();
    textures.sort();
    Ok(TextureList { textures, edited: edited_textures(&pack_dir(paths, id)?) })
}

/// The edited version of a texture when there is one, otherwise the original from the game.
pub fn read_texture(paths: &DataPaths, id: &str, path: &str) -> AppResult<Vec<u8>> {
    checked_texture_path(path)?;
    let pack = info(paths, id)?;
    if let Ok(bytes) = fs::read(texture_file(&pack_dir(paths, id)?, path)) {
        return Ok(bytes);
    }
    let mut jar = open_jar(paths, &pack.game_version)?;
    let mut entry = jar.by_name(&format!("{TEXTURES_ROOT}/{path}")).map_err(|_| invalid("this texture does not exist in the game"))?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn validate_png(bytes: &[u8]) -> AppResult<()> {
    if bytes.len() > MAX_TEXTURE_BYTES || bytes.len() < 24 || bytes[..8] != PNG_SIGNATURE {
        return Err(invalid("the texture must be a PNG image under 4 MB"));
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width == 0 || height == 0 || width > MAX_TEXTURE_SIDE || height > MAX_TEXTURE_SIDE {
        return Err(invalid("the texture has unsupported dimensions"));
    }
    Ok(())
}

pub fn write_texture(paths: &DataPaths, id: &str, path: &str, png: &[u8]) -> AppResult<()> {
    checked_texture_path(path)?;
    validate_png(png)?;
    let pack = info(paths, id)?;
    open_jar(paths, &pack.game_version)?
        .by_name(&format!("{TEXTURES_ROOT}/{path}"))
        .map_err(|_| invalid("this texture does not exist in the game"))?;
    let target = texture_file(&pack_dir(paths, id)?, path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target, png)?;
    refresh_copies(paths, id);
    Ok(())
}

pub fn reset_texture(paths: &DataPaths, id: &str, path: &str) -> AppResult<()> {
    checked_texture_path(path)?;
    match fs::remove_file(texture_file(&pack_dir(paths, id)?, path)) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error.into()),
        _ => {}
    }
    refresh_copies(paths, id);
    Ok(())
}

fn mcmeta(pack: &PackInfo) -> serde_json::Value {
    let mut section = json!({ "pack_format": pack.pack_format, "description": pack.name });
    if pack.ranged_format {
        section["min_format"] = json!(pack.pack_format);
        section["max_format"] = json!(pack.pack_format);
    }
    json!({ "pack": section })
}

/// Packs the edited textures into a resource pack zip the game can load.
pub fn export(paths: &DataPaths, id: &str, destination: &Path) -> AppResult<()> {
    let pack = info(paths, id)?;
    let dir = pack_dir(paths, id)?;
    let mut zip = zip::ZipWriter::new(File::create(destination)?);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("pack.mcmeta", options).map_err(zip_error)?;
    zip.write_all(&serde_json::to_vec_pretty(&mcmeta(&pack))?)?;
    for path in edited_textures(&dir) {
        zip.start_file(format!("{TEXTURES_ROOT}/{path}"), options).map_err(zip_error)?;
        std::io::copy(&mut File::open(texture_file(&dir, &path))?, &mut zip)?;
    }
    zip.finish().map_err(zip_error)?;
    Ok(())
}

fn file_name_for(pack: &PackInfo) -> String {
    let cleaned: String = pack
        .name
        .chars()
        .map(|character| if character.is_alphanumeric() || matches!(character, ' ' | '-' | '_') { character } else { '_' })
        .collect();
    let cleaned = cleaned.trim();
    let base = if cleaned.is_empty() { "texture-pack" } else { cleaned };
    format!("{base}-{}.zip", &pack.id[..pack.id.len().min(4)])
}

fn instance_pack_file(paths: &DataPaths, pack: &PackInfo, instance_id: &str) -> PathBuf {
    paths.instance_game_dir(instance_id).join("resourcepacks").join(file_name_for(pack))
}

/// Switches a resource pack on or off in the instance's `options.txt`, leaving every other line alone.
fn set_pack_enabled(game_dir: &Path, file_name: &str, enabled: bool) -> AppResult<()> {
    const KEY: &str = "resourcePacks:";
    let options = game_dir.join("options.txt");
    let text = fs::read_to_string(&options).unwrap_or_default();
    let line_end = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let entry = format!("file/{file_name}");
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();

    let position = lines.iter().position(|line| line.starts_with(KEY));
    let mut packs: Vec<String> = match position {
        Some(index) => serde_json::from_str(&lines[index][KEY.len()..]).map_err(|_| invalid("options.txt has an unexpected resource pack list"))?,
        None if enabled => vec!["vanilla".to_owned()],
        None => return Ok(()),
    };
    packs.retain(|pack| *pack != entry);
    if enabled {
        packs.push(entry);
    }
    let updated = format!("{KEY}{}", serde_json::to_string(&packs)?);
    match position {
        Some(index) => lines[index] = updated,
        None => lines.push(updated),
    }
    fs::create_dir_all(game_dir)?;
    fs::write(options, lines.join(line_end) + line_end)?;
    Ok(())
}

fn place_in_instance(paths: &DataPaths, pack: &PackInfo, instance_id: &str) -> AppResult<()> {
    instances::get(paths, instance_id)?;
    let target = instance_pack_file(paths, pack, instance_id);
    if let Some(folder) = target.parent() {
        fs::create_dir_all(folder)?;
    }
    export(paths, &pack.id, &target)?;
    set_pack_enabled(&paths.instance_game_dir(instance_id), &file_name_for(pack), true)
}

fn remove_from_instance(paths: &DataPaths, pack: &PackInfo, instance_id: &str) {
    let _ = fs::remove_file(instance_pack_file(paths, pack, instance_id));
    let _ = set_pack_enabled(&paths.instance_game_dir(instance_id), &file_name_for(pack), false);
}

/// Best effort: the game may be running and holding the file, in which case the next launch catches up.
fn preferences_path(paths: &DataPaths) -> PathBuf {
    paths.root().join("texture-pack-preferences.json")
}

/// Whether every pack is used in every instance of its game version, without choosing instances one by one.
pub fn everywhere(paths: &DataPaths) -> bool {
    fs::read(preferences_path(paths))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Preferences>(&bytes).ok())
        .map_or(true, |preferences| preferences.packs_everywhere)
}

fn is_used_in(pack: &PackInfo, instance: &Instance, everywhere: bool) -> bool {
    pack.instances.contains(&instance.id) || (everywhere && instance.game_version == pack.game_version)
}

/// Copies the pack into every instance that should have it and switches it on there.
fn place_where_used(paths: &DataPaths, pack: &PackInfo) {
    let everywhere = everywhere(paths);
    for instance in instances::list(paths).unwrap_or_default() {
        if is_used_in(pack, &instance, everywhere) {
            let _ = place_in_instance(paths, pack, &instance.id);
        }
    }
}

fn refresh_copies(paths: &DataPaths, id: &str) {
    if let Ok(pack) = info(paths, id) {
        place_where_used(paths, &pack);
    }
}

/// Brings every pack into the instances that should have it, e.g. when the launcher starts.
pub fn sync_all(paths: &DataPaths) {
    for pack in list(paths).unwrap_or_default() {
        place_where_used(paths, &pack);
    }
}

pub fn set_everywhere(paths: &DataPaths, enabled: bool) -> AppResult<()> {
    fs::write(preferences_path(paths), serde_json::to_vec_pretty(&Preferences { packs_everywhere: enabled })?)?;
    let installed = instances::list(paths).unwrap_or_default();
    for pack in list(paths)? {
        if enabled {
            place_where_used(paths, &pack);
        } else {
            for instance in installed.iter().filter(|instance| !pack.instances.contains(&instance.id)) {
                remove_from_instance(paths, &pack, &instance.id);
            }
        }
    }
    Ok(())
}

/// Makes exactly these instances use the pack: it is copied in and switched on, and taken away from the rest.
pub fn use_in_instances(paths: &DataPaths, id: &str, instance_ids: Vec<String>) -> AppResult<PackInfo> {
    let mut pack = info(paths, id)?;
    let mut wanted: Vec<String> = Vec::new();
    for instance_id in instance_ids {
        instances::get(paths, &instance_id)?;
        if !wanted.contains(&instance_id) {
            wanted.push(instance_id);
        }
    }
    let previous = std::mem::replace(&mut pack.instances, wanted);
    write_info(&pack_dir(paths, id)?, &pack)?;
    let everywhere = everywhere(paths);
    for removed in previous.iter().filter(|existing| !pack.instances.contains(existing)) {
        let kept = instances::get(paths, removed).is_ok_and(|instance| is_used_in(&pack, &instance, everywhere));
        if !kept {
            remove_from_instance(paths, &pack, removed);
        }
    }
    for instance_id in &pack.instances {
        place_in_instance(paths, &pack, instance_id)?;
    }
    Ok(pack)
}

/// Called before a game starts so every pack the instance uses is current and switched on.
pub fn prepare_for_launch(paths: &DataPaths, instance_id: &str) {
    let Ok(instance) = instances::get(paths, instance_id) else { return };
    let everywhere = everywhere(paths);
    for pack in list(paths).unwrap_or_default() {
        if is_used_in(&pack, &instance, everywhere) {
            let _ = place_in_instance(paths, &pack, instance_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_TEXTURE: &str = "block/stone.png";

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = PNG_SIGNATURE.to_vec();
        bytes.extend_from_slice(&[0, 0, 0, 13, b'I', b'H', b'D', b'R']);
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes
    }

    fn scratch(name: &str, version_json: &str) -> DataPaths {
        let paths = DataPaths::new(std::env::temp_dir().join(format!("kairos-packs-{name}-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        fs::create_dir_all(paths.version_dir("1.21")).unwrap();
        fs::write(paths.version_json("1.21"), "{}").unwrap();
        let mut zip = zip::ZipWriter::new(File::create(paths.client_jar("1.21")).unwrap());
        let options = SimpleFileOptions::default();
        zip.start_file("version.json", options).unwrap();
        zip.write_all(version_json.as_bytes()).unwrap();
        for entry in [FIRST_TEXTURE, "item/apple.png"] {
            zip.start_file(format!("{TEXTURES_ROOT}/{entry}"), options).unwrap();
            zip.write_all(&png(16, 16)).unwrap();
        }
        zip.start_file(format!("{TEXTURES_ROOT}/block/stone.png.mcmeta"), options).unwrap();
        zip.finish().unwrap();
        paths
    }

    #[test]
    fn reads_the_pack_format_in_every_known_shape() {
        for (text, expected) in [
            (r#"{"pack_version": 34}"#, (34, false)),
            (r#"{"pack_version": {"resource": 34, "data": 48}}"#, (34, false)),
            (r#"{"pack_version": {"resource_major": 69, "resource_minor": 0}}"#, (69, true)),
        ] {
            let paths = scratch("format", text);
            assert_eq!(pack_format_of(&mut open_jar(&paths, "1.21").unwrap()).unwrap(), expected);
            let _ = fs::remove_dir_all(paths.root());
        }
        let paths = scratch("format-missing", "{}");
        assert!(pack_format_of(&mut open_jar(&paths, "1.21").unwrap()).is_err());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn creates_edits_exports_and_resets_a_pack() {
        let paths = scratch("flow", r#"{"pack_version": 34}"#);
        assert_eq!(available_versions(&paths).unwrap(), vec!["1.21".to_owned()]);

        let pack = create(&paths, "  My pack ", "1.21").unwrap();
        assert_eq!((pack.name.as_str(), pack.pack_format), ("My pack", 34));
        assert_eq!(list(&paths).unwrap().len(), 1);

        let listing = list_textures(&paths, &pack.id).unwrap();
        assert_eq!(listing.textures, vec![FIRST_TEXTURE.to_owned(), "item/apple.png".to_owned()]);
        assert!(listing.edited.is_empty());
        assert_eq!(read_texture(&paths, &pack.id, FIRST_TEXTURE).unwrap(), png(16, 16));

        write_texture(&paths, &pack.id, FIRST_TEXTURE, &png(32, 32)).unwrap();
        assert_eq!(read_texture(&paths, &pack.id, FIRST_TEXTURE).unwrap(), png(32, 32));
        assert_eq!(list_textures(&paths, &pack.id).unwrap().edited, vec![FIRST_TEXTURE.to_owned()]);

        let archive = paths.root().join("out.zip");
        export(&paths, &pack.id, &archive).unwrap();
        let mut zip = zip::ZipArchive::new(File::open(&archive).unwrap()).unwrap();
        let mut meta = String::new();
        zip.by_name("pack.mcmeta").unwrap().read_to_string(&mut meta).unwrap();
        assert!(meta.contains("\"pack_format\": 34") && !meta.contains("min_format"));
        assert!(zip.by_name("assets/minecraft/textures/block/stone.png").is_ok());
        assert!(zip.by_name("assets/minecraft/textures/item/apple.png").is_err());

        reset_texture(&paths, &pack.id, FIRST_TEXTURE).unwrap();
        assert_eq!(read_texture(&paths, &pack.id, FIRST_TEXTURE).unwrap(), png(16, 16));
        delete(&paths, &pack.id).unwrap();
        assert!(list(&paths).unwrap().is_empty());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn ranged_formats_write_min_and_max() {
        let paths = scratch("ranged", r#"{"pack_version": {"resource_major": 69, "resource_minor": 0}}"#);
        let pack = create(&paths, "New", "1.21").unwrap();
        let archive = paths.root().join("out.zip");
        export(&paths, &pack.id, &archive).unwrap();
        let mut meta = String::new();
        zip::ZipArchive::new(File::open(&archive).unwrap()).unwrap().by_name("pack.mcmeta").unwrap().read_to_string(&mut meta).unwrap();
        assert!(meta.contains("\"min_format\": 69") && meta.contains("\"max_format\": 69"));
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn refuses_paths_and_files_that_could_do_harm() {
        let paths = scratch("harm", r#"{"pack_version": 34}"#);
        let pack = create(&paths, "Safe", "1.21").unwrap();
        for bad in ["../x.png", "block/../../x.png", "/etc/x.png", "block\\x.png", "block/Stone.png", "block/stone.txt", "", "a//b.png", "block/c:.png"] {
            assert!(write_texture(&paths, &pack.id, bad, &png(16, 16)).is_err(), "{bad}");
            assert!(read_texture(&paths, &pack.id, bad).is_err(), "{bad}");
        }
        assert!(write_texture(&paths, &pack.id, FIRST_TEXTURE, b"not a png").is_err());
        assert!(write_texture(&paths, &pack.id, FIRST_TEXTURE, &png(9000, 16)).is_err());
        assert!(write_texture(&paths, &pack.id, "block/unknown.png", &png(16, 16)).is_err());
        assert!(delete(&paths, "../x").is_err());
        assert!(create(&paths, "   ", "1.21").is_err());
        assert!(create(&paths, "x", "../1.21").is_err());
        let _ = fs::remove_dir_all(paths.root());
    }

    fn instance_for(paths: &DataPaths) -> String {
        let icon = crate::instances::model::IconInput::Preset { key: "cube".into() };
        let new = crate::instances::model::NewInstance {
            name: "Test".into(),
            game_version: "1.21".into(),
            icon,
            loader: crate::instances::model::LoaderKind::Vanilla,
            loader_version: None,
        };
        instances::create(paths, new).unwrap().id
    }

    #[test]
    fn names_pack_files_safely_and_uniquely() {
        let pack = |name: &str, id: &str| PackInfo {
            id: id.into(),
            name: name.into(),
            game_version: "1.21".into(),
            pack_format: 34,
            ranged_format: false,
            instances: Vec::new(),
            created_at: Utc::now(),
        };
        assert_eq!(file_name_for(&pack("My: pack?", "1a2b3c4d")), "My_ pack_-1a2b.zip");
        assert_eq!(file_name_for(&pack("???", "ffff0000")), "___-ffff.zip");
        assert_eq!(file_name_for(&pack("  ", "abcd1234")), "texture-pack-abcd.zip");
    }

    #[test]
    fn switches_packs_on_and_off_in_options_without_touching_other_lines() {
        let paths = scratch("options", r#"{"pack_version": 34}"#);
        let game = paths.root().join("game");
        set_pack_enabled(&game, "a.zip", true).unwrap();
        assert_eq!(fs::read_to_string(game.join("options.txt")).unwrap(), "resourcePacks:[\"vanilla\",\"file/a.zip\"]\n");

        fs::write(game.join("options.txt"), "fov:70.0\r\nresourcePacks:[\"vanilla\",\"file/other.zip\"]\r\nlang:pl_pl\r\n").unwrap();
        set_pack_enabled(&game, "a.zip", true).unwrap();
        set_pack_enabled(&game, "a.zip", true).unwrap();
        assert_eq!(
            fs::read_to_string(game.join("options.txt")).unwrap(),
            "fov:70.0\r\nresourcePacks:[\"vanilla\",\"file/other.zip\",\"file/a.zip\"]\r\nlang:pl_pl\r\n"
        );
        set_pack_enabled(&game, "a.zip", false).unwrap();
        assert_eq!(
            fs::read_to_string(game.join("options.txt")).unwrap(),
            "fov:70.0\r\nresourcePacks:[\"vanilla\",\"file/other.zip\"]\r\nlang:pl_pl\r\n"
        );

        fs::write(game.join("options.txt"), "resourcePacks:not json\n").unwrap();
        assert!(set_pack_enabled(&game, "a.zip", true).is_err());
        assert_eq!(fs::read_to_string(game.join("options.txt")).unwrap(), "resourcePacks:not json\n");
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn linked_instances_get_an_up_to_date_pack_that_is_switched_on() {
        let paths = scratch("linked", r#"{"pack_version": 34}"#);
        set_everywhere(&paths, false).unwrap();
        let instance = instance_for(&paths);
        let pack = create(&paths, "Mine", "1.21").unwrap();

        let linked = use_in_instances(&paths, &pack.id, vec![instance.clone(), instance.clone()]).unwrap();
        assert_eq!(linked.instances, vec![instance.clone()]);
        let game = paths.instance_game_dir(&instance);
        let file = game.join("resourcepacks").join(file_name_for(&linked));
        assert!(file.is_file());
        assert!(fs::read_to_string(game.join("options.txt")).unwrap().contains(&file_name_for(&linked)));

        write_texture(&paths, &pack.id, FIRST_TEXTURE, &png(32, 32)).unwrap();
        let mut zip = zip::ZipArchive::new(File::open(&file).unwrap()).unwrap();
        assert!(zip.by_name("assets/minecraft/textures/block/stone.png").is_ok());

        fs::remove_file(&file).unwrap();
        prepare_for_launch(&paths, &instance);
        assert!(file.is_file());

        use_in_instances(&paths, &pack.id, Vec::new()).unwrap();
        assert!(!file.exists());
        assert!(!fs::read_to_string(game.join("options.txt")).unwrap().contains(&file_name_for(&linked)));
        assert!(use_in_instances(&paths, &pack.id, vec!["missing".into()]).is_err());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn every_pack_is_used_in_every_instance_of_its_version_until_switched_off() {
        let paths = scratch("everywhere", r#"{"pack_version": 34}"#);
        let same = instance_for(&paths);
        let other = {
            let icon = crate::instances::model::IconInput::Preset { key: "cube".into() };
            let new = crate::instances::model::NewInstance {
                name: "Older".into(),
                game_version: "1.20".into(),
                icon,
                loader: crate::instances::model::LoaderKind::Vanilla,
                loader_version: None,
            };
            instances::create(&paths, new).unwrap().id
        };
        assert!(everywhere(&paths));

        let pack = create(&paths, "Everywhere", "1.21").unwrap();
        let file = |instance: &str| instance_pack_file(&paths, &pack, instance);
        assert!(file(&same).is_file() && !file(&other).exists());

        set_everywhere(&paths, false).unwrap();
        assert!(!everywhere(&paths) && !file(&same).exists());
        set_everywhere(&paths, true).unwrap();
        assert!(file(&same).is_file());

        use_in_instances(&paths, &pack.id, vec![other.clone()]).unwrap();
        assert!(file(&other).is_file() && file(&same).is_file());
        set_everywhere(&paths, false).unwrap();
        assert!(file(&other).is_file() && !file(&same).exists());

        fs::remove_file(file(&other)).unwrap();
        prepare_for_launch(&paths, &other);
        assert!(file(&other).is_file());

        delete(&paths, &pack.id).unwrap();
        assert!(!file(&other).exists());
        let _ = fs::remove_dir_all(paths.root());
    }
}
