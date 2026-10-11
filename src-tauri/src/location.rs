use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "location.json";
const PROBE_FILE: &str = ".zenith-write-test";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Location {
    data_dir: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MoveReport {
    pub files: u64,
    pub bytes: u64,
}

fn invalid(message: &str) -> AppError {
    AppError::InvalidInput(message.to_owned())
}

/// Where the launcher keeps games, mods, skins and everything else it downloads.
/// The choice itself is remembered in a tiny file that always stays in `config_dir`.
pub fn resolve(config_dir: &Path, default: &Path) -> PathBuf {
    fs::read(config_dir.join(CONFIG_FILE))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Location>(&bytes).ok())
        .map(|location| location.data_dir)
        .filter(|dir| dir.is_absolute() && fs::create_dir_all(dir).is_ok())
        .unwrap_or_else(|| default.to_owned())
}

fn save(config_dir: &Path, data_dir: &Path) -> AppResult<()> {
    fs::create_dir_all(config_dir)?;
    let target = config_dir.join(CONFIG_FILE);
    let temp = target.with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(&Location { data_dir: data_dir.to_owned() })?)?;
    fs::rename(temp, target)?;
    Ok(())
}

fn normalized(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").trim_end_matches('/').to_lowercase()
}

fn is_inside(path: &Path, folder: &Path) -> bool {
    let (path, folder) = (normalized(path), normalized(folder));
    path == folder || path.starts_with(&format!("{folder}/"))
}

fn copy_tree(from: &Path, to: &Path, top: bool, report: &mut MoveReport) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        if top && entry.file_name() == CONFIG_FILE {
            continue;
        }
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target, false, report)?;
        } else {
            report.bytes += fs::copy(entry.path(), target)?;
            report.files += 1;
        }
    }
    Ok(())
}

fn measure(folder: &Path) -> MoveReport {
    let mut report = MoveReport { files: 0, bytes: 0 };
    let Ok(entries) = fs::read_dir(folder) else { return report };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else { continue };
        if kind.is_dir() {
            let inner = measure(&entry.path());
            report.files += inner.files;
            report.bytes += inner.bytes;
        } else if let Ok(meta) = entry.metadata() {
            report.files += 1;
            report.bytes += meta.len();
        }
    }
    report
}

/// Total size of what the launcher stores in `folder`.
pub fn size_of(folder: &Path) -> u64 {
    measure(folder).bytes
}

fn check_target(from: &Path, to: &Path) -> AppResult<()> {
    if !to.is_absolute() {
        return Err(invalid("choose a full folder path"));
    }
    if is_inside(to, from) || is_inside(from, to) {
        return Err(invalid("the new folder cannot be inside the current one, or the other way round"));
    }
    if to.exists() && fs::read_dir(to)?.next().is_some() {
        return Err(invalid("the new folder must be empty"));
    }
    fs::create_dir_all(to)?;
    let probe = to.join(PROBE_FILE);
    fs::write(&probe, b"ok").map_err(|_| invalid("Zenith cannot write to that folder"))?;
    let _ = fs::remove_file(probe);
    Ok(())
}

/// Copies everything to the new folder, remembers it for the next start and frees the old place.
pub fn move_data(config_dir: &Path, from: &Path, to: &Path) -> AppResult<MoveReport> {
    check_target(from, to)?;
    let before = measure(from);
    let mut copied = MoveReport { files: 0, bytes: 0 };
    let result = copy_tree(from, to, true, &mut copied).map_err(AppError::from).and_then(|()| {
        let config_in_source = from.join(CONFIG_FILE);
        let source_files = before.files - u64::from(config_in_source.is_file());
        if copied.files != source_files {
            return Err(AppError::Internal("not every file was copied, nothing was changed".into()));
        }
        save(config_dir, to)
    });
    if let Err(error) = result {
        let _ = fs::remove_dir_all(to);
        return Err(error);
    }
    if let Ok(entries) = fs::read_dir(from) {
        for entry in entries.flatten() {
            if entry.file_name() == CONFIG_FILE {
                continue;
            }
            let path = entry.path();
            let _ = if path.is_dir() { fs::remove_dir_all(path) } else { fs::remove_file(path) };
        }
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("zenith-location-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn fill(folder: &Path) {
        fs::create_dir_all(folder.join("instances/one/minecraft")).unwrap();
        fs::write(folder.join("accounts.json"), b"{}").unwrap();
        fs::write(folder.join("instances/one/minecraft/options.txt"), b"fov:70").unwrap();
    }

    #[test]
    fn uses_the_default_until_a_folder_is_chosen_and_then_the_choice() {
        let root = scratch("resolve");
        let (config, default, chosen) = (root.join("config"), root.join("default"), root.join("chosen"));
        assert_eq!(resolve(&config, &default), default);

        save(&config, &chosen).unwrap();
        assert_eq!(resolve(&config, &default), chosen);
        assert!(chosen.is_dir());

        fs::write(config.join(CONFIG_FILE), b"garbage").unwrap();
        assert_eq!(resolve(&config, &default), default);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn moves_everything_keeps_the_choice_file_and_frees_the_old_folder() {
        let root = scratch("move");
        let (current, target) = (root.join("current"), root.join("elsewhere/Zenith"));
        fill(&current);
        save(&current, &current).unwrap();

        let report = move_data(&current, &current, &target).unwrap();

        assert_eq!(report, MoveReport { files: 2, bytes: 8 });
        assert_eq!(fs::read(target.join("instances/one/minecraft/options.txt")).unwrap(), b"fov:70");
        assert!(!current.join("accounts.json").exists() && !current.join("instances").exists());
        assert_eq!(resolve(&current, &current), target);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_unsafe_targets_without_touching_anything() {
        let root = scratch("refuse");
        let current = root.join("current");
        fill(&current);
        let occupied = root.join("occupied");
        fs::create_dir_all(&occupied).unwrap();
        fs::write(occupied.join("mine.txt"), b"x").unwrap();

        for target in [current.join("inside"), root.clone(), occupied, PathBuf::from("relative/folder"), current.clone()] {
            assert!(move_data(&current, &current, &target).is_err(), "{}", target.display());
        }
        assert!(current.join("accounts.json").is_file());
        assert_eq!(resolve(&current, &current), current);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn measures_the_stored_size() {
        let root = scratch("size");
        fill(&root);
        assert_eq!(size_of(&root), 8);
        let _ = fs::remove_dir_all(root);
    }
}
