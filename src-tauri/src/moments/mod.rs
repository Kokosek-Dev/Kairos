use crate::error::{AppError, AppResult};
use crate::instances::model::{InstanceSettings, LoaderKind};
use crate::instances::store;
use crate::paths::DataPaths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

const MAX_AUTOMATIC: usize = 10;
const MAX_NOTE_CHARS: usize = 120;
/// Folders of the game directory that belong to a moment. Worlds are only kept when asked for.
const SETTINGS_FOLDERS: [&str; 4] = ["mods", "config", "resourcepacks", "shaderpacks"];
const WORLDS_FOLDER: &str = "saves";
const OPTIONS_FILE: &str = "options.txt";
/// The list of installed content lives next to the game directory.
const CONTENT_INDEX: &str = "content.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Manual,
    BeforeUpdate,
    BeforeChange,
    BeforeRestore,
}

impl Reason {
    fn is_automatic(self) -> bool {
        self != Self::Manual
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub game_version: String,
    pub loader: LoaderKind,
    pub loader_version: Option<String>,
    pub memory_mb: u32,
    pub jvm_args: Vec<String>,
    pub java_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Moment {
    pub id: String,
    pub note: String,
    pub reason: Reason,
    pub created_at: DateTime<Utc>,
    pub includes_worlds: bool,
    pub size_bytes: u64,
    pub mods: Vec<String>,
    pub settings: Settings,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Difference {
    /// Mod files that are installed now but were not then.
    pub added: Vec<String>,
    /// Mod files that were installed then and are gone now.
    pub removed: Vec<String>,
    pub version_changed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sizes {
    pub settings_bytes: u64,
    pub worlds_bytes: u64,
}

fn invalid(message: &str) -> AppError {
    AppError::InvalidInput(message.to_owned())
}

fn zip_error(error: zip::result::ZipError) -> AppError {
    AppError::Internal(error.to_string())
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 40 && id.bytes().all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
}

fn moments_dir(paths: &DataPaths, instance_id: &str) -> PathBuf {
    paths.instance_dir(instance_id).join("moments")
}

fn moment_dir(paths: &DataPaths, instance_id: &str, id: &str) -> AppResult<PathBuf> {
    store::validate_id(instance_id)?;
    if !is_valid_id(id) {
        return Err(invalid("invalid moment id"));
    }
    Ok(moments_dir(paths, instance_id).join(id))
}

fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else { return 0 };
    entries
        .flatten()
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => dir_size(&entry.path()),
            Ok(kind) if kind.is_file() => entry.metadata().map_or(0, |meta| meta.len()),
            _ => 0,
        })
        .sum()
}

fn mod_names(game_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(game_dir.join("mods"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

pub fn sizes(paths: &DataPaths, instance_id: &str) -> AppResult<Sizes> {
    store::get(paths, instance_id)?;
    let game_dir = paths.instance_game_dir(instance_id);
    let settings_bytes = SETTINGS_FOLDERS.iter().map(|folder| dir_size(&game_dir.join(folder))).sum::<u64>()
        + fs::metadata(game_dir.join(OPTIONS_FILE)).map_or(0, |meta| meta.len());
    Ok(Sizes { settings_bytes, worlds_bytes: dir_size(&game_dir.join(WORLDS_FOLDER)) })
}

fn entry_options(name: &str) -> SimpleFileOptions {
    let already_compressed = name.ends_with(".jar") || name.ends_with(".disabled") || name.ends_with(".zip") || name.ends_with(".png");
    let method = if already_compressed { zip::CompressionMethod::Stored } else { zip::CompressionMethod::Deflated };
    SimpleFileOptions::default().compression_method(method).large_file(true)
}

fn add_tree(zip: &mut zip::ZipWriter<File>, folder: &Path, prefix: &str) -> AppResult<()> {
    let Ok(entries) = fs::read_dir(folder) else { return Ok(()) };
    for entry in entries.flatten() {
        let name = format!("{prefix}/{}", entry.file_name().to_string_lossy());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            add_tree(zip, &entry.path(), &name)?;
        } else if kind.is_file() {
            zip.start_file(&name, entry_options(&name)).map_err(zip_error)?;
            std::io::copy(&mut File::open(entry.path())?, zip)?;
        }
    }
    Ok(())
}

fn add_file(zip: &mut zip::ZipWriter<File>, file: &Path, name: &str) -> AppResult<()> {
    if file.is_file() {
        zip.start_file(name, entry_options(name)).map_err(zip_error)?;
        std::io::copy(&mut File::open(file)?, zip)?;
    }
    Ok(())
}

pub fn list(paths: &DataPaths, instance_id: &str) -> AppResult<Vec<Moment>> {
    store::validate_id(instance_id)?;
    let Ok(entries) = fs::read_dir(moments_dir(paths, instance_id)) else { return Ok(Vec::new()) };
    let mut moments: Vec<Moment> = entries
        .flatten()
        .filter_map(|entry| fs::read(entry.path().join("moment.json")).ok())
        .filter_map(|bytes| serde_json::from_slice(&bytes).ok())
        .collect();
    moments.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(moments)
}

fn get(paths: &DataPaths, instance_id: &str, id: &str) -> AppResult<Moment> {
    let bytes = fs::read(moment_dir(paths, instance_id, id)?.join("moment.json")).map_err(|_| invalid("this moment no longer exists"))?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn prune(paths: &DataPaths, instance_id: &str) {
    let automatic: Vec<Moment> = list(paths, instance_id).unwrap_or_default().into_iter().filter(|moment| moment.reason.is_automatic()).collect();
    for old in automatic.iter().skip(MAX_AUTOMATIC) {
        let _ = delete(paths, instance_id, &old.id);
    }
}

/// Saves the mods, settings, configuration and, when asked, the worlds of an instance so it can be brought back later.
pub fn create(paths: &DataPaths, instance_id: &str, reason: Reason, note: &str, include_worlds: bool) -> AppResult<Moment> {
    let instance = store::get(paths, instance_id)?;
    let note: String = note.trim().chars().filter(|character| !character.is_control()).take(MAX_NOTE_CHARS).collect();
    let mut id_bytes = [0u8; 2];
    getrandom::fill(&mut id_bytes).map_err(|error| AppError::Internal(error.to_string()))?;
    let created_at = Utc::now();
    let id = format!("{}-{}", created_at.format("%Y%m%d-%H%M%S"), hex::encode(id_bytes));
    let folder = moment_dir(paths, instance_id, &id)?;
    fs::create_dir_all(&folder)?;

    let game_dir = paths.instance_game_dir(instance_id);
    let build = || -> AppResult<Moment> {
        let archive = folder.join("data.zip");
        let mut zip = zip::ZipWriter::new(File::create(&archive)?);
        for name in SETTINGS_FOLDERS {
            add_tree(&mut zip, &game_dir.join(name), name)?;
        }
        if include_worlds {
            add_tree(&mut zip, &game_dir.join(WORLDS_FOLDER), WORLDS_FOLDER)?;
        }
        add_file(&mut zip, &game_dir.join(OPTIONS_FILE), OPTIONS_FILE)?;
        add_file(&mut zip, &paths.instance_dir(instance_id).join(CONTENT_INDEX), CONTENT_INDEX)?;
        zip.finish().map_err(zip_error)?;

        let moment = Moment {
            id: id.clone(),
            note,
            reason,
            created_at,
            includes_worlds: include_worlds,
            size_bytes: fs::metadata(&archive)?.len(),
            mods: mod_names(&game_dir),
            settings: Settings {
                game_version: instance.game_version.clone(),
                loader: instance.loader,
                loader_version: instance.loader_version.clone(),
                memory_mb: instance.memory_mb,
                jvm_args: instance.jvm_args.clone(),
                java_path: instance.java_path.clone(),
            },
        };
        let mut file = File::create(folder.join("moment.json"))?;
        file.write_all(&serde_json::to_vec_pretty(&moment)?)?;
        Ok(moment)
    };
    let moment = build().inspect_err(|_| {
        let _ = fs::remove_dir_all(&folder);
    })?;
    prune(paths, instance_id);
    Ok(moment)
}

pub fn delete(paths: &DataPaths, instance_id: &str, id: &str) -> AppResult<()> {
    let folder = moment_dir(paths, instance_id, id)?;
    if folder.exists() {
        fs::remove_dir_all(folder)?;
    }
    Ok(())
}

pub fn compare(paths: &DataPaths, instance_id: &str, id: &str) -> AppResult<Difference> {
    let moment = get(paths, instance_id, id)?;
    let instance = store::get(paths, instance_id)?;
    let now = mod_names(&paths.instance_game_dir(instance_id));
    Ok(Difference {
        added: now.iter().filter(|name| !moment.mods.contains(name)).cloned().collect(),
        removed: moment.mods.iter().filter(|name| !now.contains(name)).cloned().collect(),
        version_changed: moment.settings.game_version != instance.game_version
            || moment.settings.loader != instance.loader
            || moment.settings.loader_version != instance.loader_version,
    })
}

fn is_allowed_entry(first: &str, include_worlds: bool) -> bool {
    SETTINGS_FOLDERS.contains(&first) || first == OPTIONS_FILE || first == CONTENT_INDEX || (include_worlds && first == WORLDS_FOLDER)
}

/// Unpacks a moment. Only the folders and files a moment can hold are accepted, so a damaged or hostile archive cannot write elsewhere.
fn extract(archive: &Path, game_dir: &Path, instance_dir: &Path, include_worlds: bool) -> AppResult<()> {
    let mut zip = zip::ZipArchive::new(File::open(archive)?).map_err(zip_error)?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(zip_error)?;
        let relative = entry.enclosed_name().ok_or_else(|| invalid("the moment contains an unsafe path"))?;
        let first = relative.components().next().and_then(|part| part.as_os_str().to_str()).unwrap_or_default().to_owned();
        if !is_allowed_entry(&first, include_worlds) {
            return Err(invalid("the moment contains an unexpected file"));
        }
        if entry.is_dir() {
            continue;
        }
        let target = if first == CONTENT_INDEX { instance_dir.join(CONTENT_INDEX) } else { game_dir.join(&relative) };
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        std::io::copy(&mut entry, &mut File::create(target)?)?;
    }
    Ok(())
}

/// Brings the instance back to a moment. What is there now is saved as a moment first, so going back can itself be undone.
pub fn restore(paths: &DataPaths, instance_id: &str, id: &str) -> AppResult<()> {
    let moment = get(paths, instance_id, id)?;
    let archive = moment_dir(paths, instance_id, id)?.join("data.zip");
    create(paths, instance_id, Reason::BeforeRestore, "", moment.includes_worlds)?;

    let game_dir = paths.instance_game_dir(instance_id);
    for folder in SETTINGS_FOLDERS {
        let _ = fs::remove_dir_all(game_dir.join(folder));
    }
    if moment.includes_worlds {
        let _ = fs::remove_dir_all(game_dir.join(WORLDS_FOLDER));
    }
    let _ = fs::remove_file(game_dir.join(OPTIONS_FILE));
    let _ = fs::remove_file(paths.instance_dir(instance_id).join(CONTENT_INDEX));
    extract(&archive, &game_dir, &paths.instance_dir(instance_id), moment.includes_worlds)?;

    let current = store::get(paths, instance_id)?;
    let settings = moment.settings;
    store::update(
        paths,
        instance_id,
        InstanceSettings {
            name: current.name,
            game_version: settings.game_version,
            loader: settings.loader,
            loader_version: settings.loader_version,
            memory_mb: settings.memory_mb,
            jvm_args: settings.jvm_args,
            java_path: settings.java_path,
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::model::{IconInput, NewInstance};

    fn scratch(name: &str) -> (DataPaths, String) {
        let paths = DataPaths::new(std::env::temp_dir().join(format!("zenith-moments-{name}-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        let new = NewInstance {
            name: "Moments".into(),
            game_version: "1.21".into(),
            icon: IconInput::Preset { key: "cube".into() },
            loader: LoaderKind::Vanilla,
            loader_version: None,
        };
        let id = store::create(&paths, new).unwrap().id;
        let game = paths.instance_game_dir(&id);
        for (path, text) in [("mods/a.jar", "A"), ("config/x.toml", "x=1"), (OPTIONS_FILE, "fov:70"), ("saves/World/level.dat", "world")] {
            let target = game.join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(target, text).unwrap();
        }
        fs::write(paths.instance_dir(&id).join(CONTENT_INDEX), "{\"v\":1}").unwrap();
        (paths, id)
    }

    fn change_everything(paths: &DataPaths, id: &str) {
        let game = paths.instance_game_dir(id);
        fs::remove_file(game.join("mods/a.jar")).unwrap();
        fs::write(game.join("mods/b.jar"), "B").unwrap();
        fs::write(game.join(OPTIONS_FILE), "fov:110").unwrap();
        fs::write(paths.instance_dir(id).join(CONTENT_INDEX), "{\"v\":2}").unwrap();
        let current = store::get(paths, id).unwrap();
        let settings = InstanceSettings {
            name: current.name,
            game_version: "1.21.1".into(),
            loader: LoaderKind::Vanilla,
            loader_version: None,
            memory_mb: 6144,
            jvm_args: vec!["-Xss4m".into()],
            java_path: None,
        };
        store::update(paths, id, settings).unwrap();
    }

    #[test]
    fn goes_back_to_a_moment_and_back_again() {
        let (paths, id) = scratch("flow");
        let game = paths.instance_game_dir(&id);
        let before = create(&paths, &id, Reason::Manual, "  before the big change  ", false).unwrap();
        assert_eq!((before.note.as_str(), before.mods.clone()), ("before the big change", vec!["a.jar".to_owned()]));
        assert!(before.size_bytes > 0 && !before.includes_worlds);

        change_everything(&paths, &id);
        let difference = compare(&paths, &id, &before.id).unwrap();
        assert_eq!((difference.added, difference.removed, difference.version_changed), (vec!["b.jar".to_owned()], vec!["a.jar".to_owned()], true));

        restore(&paths, &id, &before.id).unwrap();
        assert_eq!(fs::read_to_string(game.join("mods/a.jar")).unwrap(), "A");
        assert!(!game.join("mods/b.jar").exists());
        assert_eq!(fs::read_to_string(game.join(OPTIONS_FILE)).unwrap(), "fov:70");
        assert_eq!(fs::read_to_string(paths.instance_dir(&id).join(CONTENT_INDEX)).unwrap(), "{\"v\":1}");
        let instance = store::get(&paths, &id).unwrap();
        assert_eq!((instance.game_version.as_str(), instance.memory_mb, instance.jvm_args.len()), ("1.21", 2048, 0));
        assert_eq!(fs::read_to_string(game.join("saves/World/level.dat")).unwrap(), "world");

        let moments = list(&paths, &id).unwrap();
        let safety = moments.iter().find(|moment| moment.reason == Reason::BeforeRestore).expect("restoring saves the present first");
        restore(&paths, &id, &safety.id).unwrap();
        assert!(game.join("mods/b.jar").exists() && !game.join("mods/a.jar").exists());
        assert_eq!(store::get(&paths, &id).unwrap().memory_mb, 6144);
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn worlds_only_come_back_when_they_were_saved() {
        let (paths, id) = scratch("worlds");
        let world = paths.instance_game_dir(&id).join("saves/World/level.dat");
        let with = create(&paths, &id, Reason::Manual, "", true).unwrap();
        let without = create(&paths, &id, Reason::Manual, "", false).unwrap();
        assert!(with.size_bytes >= without.size_bytes && with.includes_worlds);

        fs::remove_dir_all(paths.instance_game_dir(&id).join("saves")).unwrap();
        restore(&paths, &id, &without.id).unwrap();
        assert!(!world.exists());
        restore(&paths, &id, &with.id).unwrap();
        assert_eq!(fs::read_to_string(&world).unwrap(), "world");
        let report = sizes(&paths, &id).unwrap();
        assert!(report.settings_bytes > 0 && report.worlds_bytes > 0);
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn keeps_only_the_latest_automatic_moments_and_every_manual_one() {
        let (paths, id) = scratch("prune");
        let manual = create(&paths, &id, Reason::Manual, "keep me", false).unwrap();
        for _ in 0..(MAX_AUTOMATIC + 3) {
            create(&paths, &id, Reason::BeforeUpdate, "", false).unwrap();
        }
        let moments = list(&paths, &id).unwrap();
        assert_eq!(moments.iter().filter(|moment| moment.reason.is_automatic()).count(), MAX_AUTOMATIC);
        assert!(moments.iter().any(|moment| moment.id == manual.id));
        delete(&paths, &id, &manual.id).unwrap();
        assert!(!list(&paths, &id).unwrap().iter().any(|moment| moment.id == manual.id));
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn refuses_archives_that_try_to_write_elsewhere() {
        let (paths, id) = scratch("hostile");
        let archive = paths.root().join("evil.zip");
        let mut zip = zip::ZipWriter::new(File::create(&archive).unwrap());
        zip.start_file("../evil.txt", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"x").unwrap();
        zip.finish().unwrap();
        let game = paths.instance_game_dir(&id);
        assert!(extract(&archive, &game, &paths.instance_dir(&id), false).is_err());
        assert!(!paths.instance_dir(&id).join("evil.txt").exists());

        let stray = paths.root().join("stray.zip");
        let mut zip = zip::ZipWriter::new(File::create(&stray).unwrap());
        zip.start_file("instance.json", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"{}").unwrap();
        zip.start_file("saves/World/level.dat", SimpleFileOptions::default()).unwrap();
        zip.finish().unwrap();
        assert!(extract(&stray, &game, &paths.instance_dir(&id), false).is_err());
        assert!(get(&paths, &id, "../x").is_err() && delete(&paths, &id, "..").is_err());
        let _ = fs::remove_dir_all(paths.root());
    }
}
