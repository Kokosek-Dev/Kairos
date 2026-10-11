use crate::error::{AppError, AppResult};
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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureList {
    pub textures: Vec<String>,
    pub edited: Vec<String>,
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
        created_at: Utc::now(),
    };
    let dir = pack_dir(paths, &info.id)?;
    fs::create_dir_all(&dir)?;
    write_info(&dir, &info)?;
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
    Ok(())
}

pub fn reset_texture(paths: &DataPaths, id: &str, path: &str) -> AppResult<()> {
    checked_texture_path(path)?;
    match fs::remove_file(texture_file(&pack_dir(paths, id)?, path)) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
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

fn file_name_for(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|character| if character.is_alphanumeric() || matches!(character, ' ' | '-' | '_') { character } else { '_' })
        .collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() { "texture-pack".to_owned() } else { cleaned.to_owned() }
}

/// Puts the pack into an instance's resource pack folder, ready to be switched on in the game.
pub fn install(paths: &DataPaths, id: &str, instance_id: &str) -> AppResult<PathBuf> {
    let pack = info(paths, id)?;
    instances::get(paths, instance_id)?;
    let folder = paths.instance_game_dir(instance_id).join("resourcepacks");
    fs::create_dir_all(&folder)?;
    let target = folder.join(format!("{}.zip", file_name_for(&pack.name)));
    export(paths, id, &target)?;
    Ok(target)
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
        let paths = DataPaths::new(std::env::temp_dir().join(format!("zenith-packs-{name}-{}", std::process::id())));
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

    #[test]
    fn names_pack_files_safely() {
        assert_eq!(file_name_for("My: pack?"), "My_ pack_");
        assert_eq!(file_name_for("???"), "___");
        assert_eq!(file_name_for("  "), "texture-pack");
    }
}
