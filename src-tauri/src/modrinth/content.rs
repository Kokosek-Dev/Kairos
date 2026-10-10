use crate::error::{AppError, AppResult};
use crate::paths::DataPaths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const DISABLED_SUFFIX: &str = ".disabled";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    Mod,
    ResourcePack,
    Shader,
}

impl ContentKind {
    pub const ALL: [ContentKind; 3] = [Self::Mod, Self::ResourcePack, Self::Shader];

    pub fn folder(self) -> &'static str {
        match self {
            Self::Mod => "mods",
            Self::ResourcePack => "resourcepacks",
            Self::Shader => "shaderpacks",
        }
    }

    pub fn from_project_type(project_type: &str) -> Option<Self> {
        match project_type {
            "mod" => Some(Self::Mod),
            "resourcepack" => Some(Self::ResourcePack),
            "shader" => Some(Self::Shader),
            _ => None,
        }
    }

}

/// What Zenith knows about a file it installed. Files dropped into the folders by hand
/// have no entry and are listed by file name only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentEntry {
    pub kind: ContentKind,
    pub file_name: String,
    pub title: String,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub version_number: Option<String>,
    pub icon_url: Option<String>,
    pub sha512: Option<String>,
    pub download_url: Option<String>,
    pub size: Option<u64>,
    pub client_side: Option<String>,
    pub server_side: Option<String>,
    pub dependency: bool,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentItem {
    pub kind: ContentKind,
    pub file_name: String,
    pub enabled: bool,
    pub size: u64,
    pub entry: Option<ContentEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ContentIndex {
    entries: Vec<ContentEntry>,
}

pub fn is_safe_file_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.starts_with('.')
        && !name.chars().any(|character| matches!(character, '/' | '\\' | ':' | '\0') || character.is_control())
}

fn index_path(paths: &DataPaths, instance_id: &str) -> std::path::PathBuf {
    paths.instance_dir(instance_id).join("content.json")
}

pub fn load_entries(paths: &DataPaths, instance_id: &str) -> AppResult<Vec<ContentEntry>> {
    match fs::read(index_path(paths, instance_id)) {
        Ok(bytes) => Ok(serde_json::from_slice::<ContentIndex>(&bytes)?.entries),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

pub fn save_entries(paths: &DataPaths, instance_id: &str, entries: Vec<ContentEntry>) -> AppResult<()> {
    let target = index_path(paths, instance_id);
    let temp = target.with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(&ContentIndex { entries })?)?;
    fs::rename(temp, target)?;
    Ok(())
}

fn folder(paths: &DataPaths, instance_id: &str, kind: ContentKind) -> std::path::PathBuf {
    paths.instance_game_dir(instance_id).join(kind.folder())
}

pub fn list(paths: &DataPaths, instance_id: &str) -> AppResult<Vec<ContentItem>> {
    let entries = load_entries(paths, instance_id)?;
    let mut items = Vec::new();
    for kind in ContentKind::ALL {
        let Ok(directory) = fs::read_dir(folder(paths, instance_id, kind)) else {
            continue;
        };
        for file in directory.flatten() {
            let Ok(metadata) = file.metadata() else { continue };
            let name = file.file_name().to_string_lossy().into_owned();
            if !metadata.is_file() || name.starts_with('.') {
                continue;
            }
            let (file_name, enabled) = match name.strip_suffix(DISABLED_SUFFIX) {
                Some(stripped) => (stripped.to_owned(), false),
                None => (name, true),
            };
            let entry = entries.iter().find(|entry| entry.kind == kind && entry.file_name == file_name).cloned();
            items.push(ContentItem { kind, file_name, enabled, size: metadata.len(), entry });
        }
    }
    items.sort_by(|a, b| {
        let title = |item: &ContentItem| item.entry.as_ref().map_or(item.file_name.to_lowercase(), |entry| entry.title.to_lowercase());
        (a.kind as u8, title(a)).cmp(&(b.kind as u8, title(b)))
    });
    Ok(items)
}

fn require_safe(file_name: &str) -> AppResult<()> {
    is_safe_file_name(file_name)
        .then_some(())
        .ok_or_else(|| AppError::InvalidInput("invalid file name".into()))
}

pub fn set_enabled(paths: &DataPaths, instance_id: &str, kind: ContentKind, file_name: &str, enabled: bool) -> AppResult<()> {
    require_safe(file_name)?;
    let directory = folder(paths, instance_id, kind);
    let active = directory.join(file_name);
    let disabled = directory.join(format!("{file_name}{DISABLED_SUFFIX}"));
    let (from, to) = if enabled { (disabled, active) } else { (active, disabled) };
    if from.exists() {
        fs::rename(from, to)?;
    }
    Ok(())
}

pub fn remove(paths: &DataPaths, instance_id: &str, kind: ContentKind, file_name: &str) -> AppResult<()> {
    require_safe(file_name)?;
    remove_files(&folder(paths, instance_id, kind), file_name);
    let mut entries = load_entries(paths, instance_id)?;
    entries.retain(|entry| !(entry.kind == kind && entry.file_name == file_name));
    save_entries(paths, instance_id, entries)
}

pub fn remove_files(directory: &Path, file_name: &str) {
    let _ = fs::remove_file(directory.join(file_name));
    let _ = fs::remove_file(directory.join(format!("{file_name}{DISABLED_SUFFIX}")));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(kind: ContentKind, file_name: &str, title: &str) -> ContentEntry {
        ContentEntry {
            kind,
            file_name: file_name.into(),
            title: title.into(),
            project_id: Some("P".into()),
            version_id: Some("V".into()),
            version_number: Some("1.0".into()),
            icon_url: None,
            sha512: None,
            download_url: None,
            size: None,
            client_side: None,
            server_side: None,
            dependency: false,
            installed_at: Utc::now(),
        }
    }

    fn scratch(name: &str) -> DataPaths {
        let paths = DataPaths::new(std::env::temp_dir().join(format!("zenith-content-{name}-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        fs::create_dir_all(paths.instance_dir("pack")).unwrap();
        paths
    }

    #[test]
    fn lists_known_unknown_and_disabled_files() {
        let paths = scratch("list");
        let mods = paths.instance_game_dir("pack").join("mods");
        fs::create_dir_all(&mods).unwrap();
        fs::write(mods.join("sodium.jar"), b"1234").unwrap();
        fs::write(mods.join("hand-made.jar.disabled"), b"12").unwrap();
        save_entries(&paths, "pack", vec![entry(ContentKind::Mod, "sodium.jar", "Sodium")]).unwrap();

        let items = list(&paths, "pack").unwrap();

        assert_eq!(items.len(), 2);
        let sodium = items.iter().find(|item| item.file_name == "sodium.jar").unwrap();
        assert!(sodium.enabled && sodium.entry.is_some() && sodium.size == 4);
        let manual = items.iter().find(|item| item.file_name == "hand-made.jar").unwrap();
        assert!(!manual.enabled && manual.entry.is_none());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn disables_enables_and_removes_files_and_their_index_entries() {
        let paths = scratch("toggle");
        let mods = paths.instance_game_dir("pack").join("mods");
        fs::create_dir_all(&mods).unwrap();
        fs::write(mods.join("a.jar"), b"x").unwrap();
        save_entries(&paths, "pack", vec![entry(ContentKind::Mod, "a.jar", "A")]).unwrap();

        set_enabled(&paths, "pack", ContentKind::Mod, "a.jar", false).unwrap();
        assert!(mods.join("a.jar.disabled").exists() && !mods.join("a.jar").exists());
        set_enabled(&paths, "pack", ContentKind::Mod, "a.jar", true).unwrap();
        assert!(mods.join("a.jar").exists());

        remove(&paths, "pack", ContentKind::Mod, "a.jar").unwrap();
        assert!(!mods.join("a.jar").exists());
        assert!(load_entries(&paths, "pack").unwrap().is_empty());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn refuses_file_names_that_could_leave_the_folder() {
        let paths = scratch("unsafe");
        for name in ["../x.jar", "a/b.jar", "..", ".hidden", "", "c:evil.jar", "a\\b.jar"] {
            assert!(set_enabled(&paths, "pack", ContentKind::Mod, name, false).is_err(), "{name:?}");
            assert!(remove(&paths, "pack", ContentKind::Mod, name).is_err(), "{name:?}");
        }
        assert!(is_safe_file_name("sodium-fabric-0.5.jar"));
    }
}
