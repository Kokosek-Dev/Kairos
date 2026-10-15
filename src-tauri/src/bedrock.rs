use crate::error::{AppError, AppResult};
use crate::paths::DataPaths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

const MAX_NAME: usize = 48;
const MAX_INSTANCES: usize = 100;
const MAX_UNPACKED_BYTES: u64 = 512 * 1024 * 1024;
const MAX_FILES: usize = 20_000;
/// Everything Kairos puts into the game's pack folders starts with this, so it never touches packs it did not place.
const PREFIX: &str = "kairos-";
/// The readable name of an imported pack, kept because many manifests only hold a translation key.
const NAME_FILE: &str = "kairos-name.txt";
const UWP_PACKAGE: &str = "Microsoft.MinecraftUWP_8wekyb3d8bbwe";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackKind {
    Resources,
    Behavior,
}

impl PackKind {
    /// The folder of the game that lists a pack without importing it.
    fn game_folder(self) -> &'static str {
        match self {
            Self::Resources => "development_resource_packs",
            Self::Behavior => "development_behavior_packs",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stored {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockPack {
    pub folder: String,
    pub name: String,
    pub kind: PackKind,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockInstance {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub packs: Vec<BedrockPack>,
}

struct Manifest {
    name: String,
    uuid: String,
    version: String,
    kind: PackKind,
}

fn invalid(message: &str) -> AppError {
    AppError::InvalidInput(message.to_owned())
}

fn zip_error(error: zip::result::ZipError) -> AppError {
    AppError::Internal(error.to_string())
}

fn is_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn instance_dir(paths: &DataPaths, id: &str) -> AppResult<PathBuf> {
    if !is_id(id) {
        return Err(invalid("invalid Bedrock instance id"));
    }
    Ok(paths.bedrock_instances().join(id))
}

fn packs_dir(paths: &DataPaths, id: &str) -> AppResult<PathBuf> {
    Ok(instance_dir(paths, id)?.join("packs"))
}

fn read_stored(dir: &Path) -> Option<Stored> {
    serde_json::from_slice(&fs::read(dir.join("instance.json")).ok()?).ok()
}

fn clean_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME {
        return Err(invalid("invalid instance name"));
    }
    Ok(name.to_owned())
}

/// Reads the part of a pack's `manifest.json` Kairos needs. A pack that is neither a resource nor a behavior pack is not usable here.
fn parse_manifest(text: &str, fallback_name: &str) -> Option<Manifest> {
    let value: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    let header = &value["header"];
    let uuid = header["uuid"].as_str()?.to_ascii_lowercase();
    if uuid.len() != 36 || !uuid.bytes().all(|byte| byte.is_ascii_hexdigit() || byte == b'-') {
        return None;
    }
    let kinds: Vec<&str> = value["modules"].as_array()?.iter().filter_map(|module| module["type"].as_str()).collect();
    let kind = if kinds.contains(&"resources") {
        PackKind::Resources
    } else if kinds.iter().any(|kind| matches!(*kind, "data" | "script" | "client_data")) {
        PackKind::Behavior
    } else {
        return None;
    };
    // Many packs name themselves by a translation key such as `pack.name`.
    let name = header["name"].as_str().map(str::trim).filter(|name| !name.is_empty() && !name.starts_with("pack.")).unwrap_or(fallback_name);
    let version = header["version"].as_array().map_or_else(
        || "1.0.0".to_owned(),
        |parts| parts.iter().map(|part| part.as_u64().unwrap_or(0).to_string()).collect::<Vec<_>>().join("."),
    );
    Some(Manifest { name: name.chars().take(MAX_NAME).collect(), uuid, version, kind })
}

fn folder_for(manifest: &Manifest) -> String {
    let readable: String = manifest.name.chars().map(|character| if character.is_ascii_alphanumeric() { character } else { '_' }).collect();
    format!("{}-{}", readable.trim_matches('_'), &manifest.uuid[..8])
}

fn pack_from(dir: &Path, folder: &str) -> Option<BedrockPack> {
    let manifest = parse_manifest(&fs::read_to_string(dir.join("manifest.json")).ok()?, folder)?;
    let name = fs::read_to_string(dir.join(NAME_FILE)).ok().map(|text| text.trim().to_owned()).filter(|text| !text.is_empty()).unwrap_or(manifest.name);
    Some(BedrockPack { folder: folder.to_owned(), name, kind: manifest.kind, version: manifest.version })
}

fn packs_of(paths: &DataPaths, id: &str) -> Vec<BedrockPack> {
    let Ok(dir) = packs_dir(paths, id) else { return Vec::new() };
    let mut packs: Vec<BedrockPack> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| pack_from(&entry.path(), &entry.file_name().to_string_lossy()))
        .collect();
    packs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    packs
}

pub fn list(paths: &DataPaths) -> AppResult<Vec<BedrockInstance>> {
    let mut instances: Vec<BedrockInstance> = fs::read_dir(paths.bedrock_instances())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| read_stored(&entry.path()))
        .filter(|stored| is_id(&stored.id))
        .map(|stored| BedrockInstance { packs: packs_of(paths, &stored.id), id: stored.id, name: stored.name, created_at: stored.created_at })
        .collect();
    instances.sort_by_key(|instance| instance.created_at);
    Ok(instances)
}

fn get(paths: &DataPaths, id: &str) -> AppResult<BedrockInstance> {
    list(paths)?.into_iter().find(|instance| instance.id == id).ok_or_else(|| invalid("this Bedrock instance does not exist"))
}

pub fn create(paths: &DataPaths, name: &str) -> AppResult<BedrockInstance> {
    let name = clean_name(name)?;
    let existing = list(paths)?;
    if existing.len() >= MAX_INSTANCES {
        return Err(invalid("too many Bedrock instances"));
    }
    let mut stamp = Utc::now().timestamp_millis();
    while existing.iter().any(|instance| instance.id == format!("{stamp:x}")) {
        stamp += 1;
    }
    let stored = Stored { id: format!("{stamp:x}"), name, created_at: Utc::now() };
    let dir = instance_dir(paths, &stored.id)?;
    fs::create_dir_all(dir.join("packs"))?;
    fs::write(dir.join("instance.json"), serde_json::to_vec_pretty(&stored)?)?;
    get(paths, &stored.id)
}

pub fn delete(paths: &DataPaths, id: &str) -> AppResult<()> {
    match fs::remove_dir_all(instance_dir(paths, id)?) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}

/// Where the game keeps its packs and worlds: the current Microsoft Store version, or the older packaged one.
pub fn game_data_dir() -> Option<PathBuf> {
    let current = std::env::var_os("APPDATA").map(|roaming| PathBuf::from(roaming).join("Minecraft Bedrock").join("Users").join("Shared").join("games").join("com.mojang"));
    let packaged = std::env::var_os("LOCALAPPDATA").map(|local| PathBuf::from(local).join("Packages").join(UWP_PACKAGE).join("LocalState").join("games").join("com.mojang"));
    [current, packaged].into_iter().flatten().find(|dir| dir.is_dir())
}

/// Copies the packs of an archive (`.mcpack`, `.mcaddon` or a plain zip) into the instance.
pub fn add_pack_file(paths: &DataPaths, id: &str, file: &Path) -> AppResult<BedrockInstance> {
    get(paths, id)?;
    let target_root = packs_dir(paths, id)?;
    let mut archive = zip::ZipArchive::new(File::open(file)?).map_err(|_| invalid("this is not a Bedrock pack file"))?;
    let fallback = file.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();

    let mut found: Vec<(String, Manifest)> = Vec::new();
    for index in 0..archive.len() {
        let name = archive.by_index(index).map_err(zip_error)?.name().map_err(zip_error)?.into_owned();
        let Some(prefix) = name.strip_suffix("manifest.json") else { continue };
        if prefix.matches('/').count() > 1 || (!prefix.is_empty() && !prefix.ends_with('/')) {
            continue;
        }
        let mut text = String::new();
        io::Read::read_to_string(&mut archive.by_index(index).map_err(zip_error)?, &mut text).ok();
        let inside = prefix.trim_end_matches('/');
        if let Some(manifest) = parse_manifest(&text, if inside.is_empty() { &fallback } else { inside }) {
            found.push((prefix.to_owned(), manifest));
        }
    }
    // A manifest at the top means the whole archive is one pack.
    if found.iter().any(|(prefix, _)| prefix.is_empty()) {
        found.retain(|(prefix, _)| prefix.is_empty());
    }
    if found.is_empty() {
        return Err(invalid("no resource or behavior pack was found in this file"));
    }

    let mut unpacked = 0u64;
    let mut files = 0usize;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(zip_error)?;
        if found.iter().any(|(prefix, _)| entry.name().is_ok_and(|name| name.starts_with(prefix.as_str()))) {
            unpacked += entry.size();
            files += 1;
        }
    }
    if unpacked > MAX_UNPACKED_BYTES || files > MAX_FILES {
        return Err(invalid("this pack is too large"));
    }

    for (prefix, manifest) in &found {
        let destination = target_root.join(folder_for(manifest));
        let _ = fs::remove_dir_all(&destination);
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(zip_error)?;
            let Some(relative) = entry.enclosed_name().and_then(|path| path.strip_prefix(prefix.trim_end_matches('/')).ok().map(Path::to_path_buf)) else { continue };
            if !entry.name().is_ok_and(|name| name.starts_with(prefix.as_str())) || relative.as_os_str().is_empty() {
                continue;
            }
            let output = destination.join(relative);
            if entry.is_dir() {
                fs::create_dir_all(&output)?;
            } else {
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent)?;
                }
                io::copy(&mut entry, &mut File::create(output)?)?;
            }
        }
        fs::write(destination.join(NAME_FILE), &manifest.name)?;
    }
    get(paths, id)
}

pub fn remove_pack(paths: &DataPaths, id: &str, folder: &str) -> AppResult<BedrockInstance> {
    let plain = !folder.is_empty() && folder.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
    if !plain {
        return Err(invalid("invalid pack folder"));
    }
    match fs::remove_dir_all(packs_dir(paths, id)?.join(folder)) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error.into()),
        _ => {}
    }
    get(paths, id)
}

fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)?.flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn remove_placed(game_dir: &Path) {
    for kind in [PackKind::Resources, PackKind::Behavior] {
        for entry in fs::read_dir(game_dir.join(kind.game_folder())).into_iter().flatten().flatten() {
            if entry.file_name().to_string_lossy().starts_with(PREFIX) {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }
}

/// Makes the game list exactly the packs of this instance, next to whatever the player has put there themselves.
pub fn apply(paths: &DataPaths, id: &str, game_dir: &Path) -> AppResult<()> {
    let instance = get(paths, id)?;
    remove_placed(game_dir);
    for pack in &instance.packs {
        let from = packs_dir(paths, id)?.join(&pack.folder);
        copy_tree(&from, &game_dir.join(pack.kind.game_folder()).join(format!("{PREFIX}{}", pack.folder)))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn scratch(name: &str) -> DataPaths {
        let paths = DataPaths::new(std::env::temp_dir().join(format!("kairos-bedrock-{name}-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        fs::create_dir_all(paths.root()).unwrap();
        paths
    }

    fn manifest(name: &str, uuid: &str, module: &str) -> String {
        format!(r#"{{"format_version":2,"header":{{"name":"{name}","uuid":"{uuid}","version":[1,2,3]}},"modules":[{{"type":"{module}","uuid":"{uuid}","version":[1,0,0]}}]}}"#)
    }

    fn archive(paths: &DataPaths, file: &str, entries: &[(&str, &str)]) -> PathBuf {
        let path = paths.root().join(file);
        let mut writer = zip::ZipWriter::new(File::create(&path).unwrap());
        for (name, text) in entries {
            writer.start_file(*name, SimpleFileOptions::default()).unwrap();
            writer.write_all(text.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
        path
    }

    const RESOURCES: &str = "11111111-2222-3333-4444-555555555555";
    const BEHAVIOR: &str = "66666666-7777-8888-9999-000000000000";

    #[test]
    fn creates_lists_and_deletes_instances() {
        let paths = scratch("crud");
        assert!(create(&paths, "  ").is_err());
        let first = create(&paths, "  Survival  ").unwrap();
        let second = create(&paths, "Creative").unwrap();
        assert_ne!(first.id, second.id);
        assert_eq!(list(&paths).unwrap().iter().map(|instance| instance.name.as_str()).collect::<Vec<_>>(), ["Survival", "Creative"]);

        delete(&paths, &first.id).unwrap();
        assert_eq!(list(&paths).unwrap().len(), 1);
        assert!(delete(&paths, "../escape").is_err());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn takes_both_packs_out_of_an_addon_file() {
        let paths = scratch("addon");
        let instance = create(&paths, "Addons").unwrap();
        let file = archive(
            &paths,
            "cool.mcaddon",
            &[
                ("RP/manifest.json", &manifest("pack.name", RESOURCES, "resources")),
                ("RP/textures/block.png", "png"),
                ("BP/manifest.json", &manifest("Cool behavior", BEHAVIOR, "data")),
                ("BP/entities/a.json", "{}"),
            ],
        );

        let result = add_pack_file(&paths, &instance.id, &file).unwrap();

        assert_eq!(result.packs.len(), 2);
        let resources = result.packs.iter().find(|pack| pack.kind == PackKind::Resources).unwrap();
        assert_eq!((resources.name.as_str(), resources.version.as_str()), ("RP", "1.2.3"));
        let behavior = result.packs.iter().find(|pack| pack.kind == PackKind::Behavior).unwrap();
        assert_eq!(behavior.name, "Cool behavior");
        assert!(packs_dir(&paths, &instance.id).unwrap().join(&resources.folder).join("textures/block.png").is_file());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn a_single_pack_file_has_its_manifest_at_the_top() {
        let paths = scratch("single");
        let instance = create(&paths, "One").unwrap();
        let file = archive(&paths, "Fancy Sky.mcpack", &[("manifest.json", &manifest("pack.name", RESOURCES, "resources")), ("pack_icon.png", "x")]);

        let result = add_pack_file(&paths, &instance.id, &file).unwrap();

        assert_eq!(result.packs.len(), 1);
        assert_eq!(result.packs[0].name, "Fancy Sky");
        let removed = remove_pack(&paths, &instance.id, &result.packs[0].folder).unwrap();
        assert!(removed.packs.is_empty());
        assert!(remove_pack(&paths, &instance.id, "../x").is_err());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn refuses_files_without_a_usable_pack_and_never_writes_outside() {
        let paths = scratch("refuse");
        let instance = create(&paths, "Safe").unwrap();
        let skins = archive(&paths, "skins.mcpack", &[("manifest.json", &manifest("Skins", RESOURCES, "skin_pack"))]);
        let nothing = archive(&paths, "empty.zip", &[("readme.txt", "hi")]);
        let hostile = archive(
            &paths,
            "hostile.mcaddon",
            &[("RP/manifest.json", &manifest("Evil", RESOURCES, "resources")), ("RP/../../../outside.txt", "gotcha")],
        );
        let not_a_zip = paths.root().join("plain.mcpack");
        fs::write(&not_a_zip, "not a zip").unwrap();

        assert!(add_pack_file(&paths, &instance.id, &skins).is_err());
        assert!(add_pack_file(&paths, &instance.id, &nothing).is_err());
        assert!(add_pack_file(&paths, &instance.id, &not_a_zip).is_err());
        add_pack_file(&paths, &instance.id, &hostile).unwrap();
        assert!(!paths.root().join("outside.txt").exists() && !paths.bedrock_instances().join("outside.txt").exists());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn puts_only_the_packs_of_the_instance_into_the_game_and_keeps_the_players_own() {
        let paths = scratch("apply");
        let game = paths.root().join("com.mojang");
        fs::create_dir_all(game.join("development_resource_packs/mine")).unwrap();
        fs::write(game.join("development_resource_packs/mine/manifest.json"), "{}").unwrap();
        fs::create_dir_all(game.join("development_behavior_packs/kairos-old-1234")).unwrap();

        let instance = create(&paths, "Play").unwrap();
        let file = archive(&paths, "a.mcaddon", &[("RP/manifest.json", &manifest("Look", RESOURCES, "resources")), ("RP/a.txt", "a"), ("BP/manifest.json", &manifest("Logic", BEHAVIOR, "data"))]);
        let packs = add_pack_file(&paths, &instance.id, &file).unwrap().packs;

        apply(&paths, &instance.id, &game).unwrap();

        assert!(game.join("development_resource_packs/mine/manifest.json").is_file());
        assert!(!game.join("development_behavior_packs/kairos-old-1234").exists());
        for pack in &packs {
            assert!(game.join(pack.kind.game_folder()).join(format!("kairos-{}", pack.folder)).join("manifest.json").is_file());
        }
        let other = create(&paths, "Empty").unwrap();
        apply(&paths, &other.id, &game).unwrap();
        assert!(game.join("development_resource_packs/mine").is_dir());
        assert_eq!(fs::read_dir(game.join("development_behavior_packs")).unwrap().count(), 0);
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn reads_manifests_with_a_byte_order_mark() {
        let text = format!("\u{feff}{}", manifest("Marked", RESOURCES, "resources"));
        assert_eq!(parse_manifest(&text, "x").unwrap().name, "Marked");
        assert!(parse_manifest("{}", "x").is_none());
    }
}
