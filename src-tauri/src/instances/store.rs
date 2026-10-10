use super::model::{
    IconInput, Instance, InstanceIcon, InstanceSettings, LoaderKind, NewInstance, DEFAULT_MEMORY_MB, SCHEMA_VERSION,
};
use crate::error::{AppError, AppResult};
use crate::paths::DataPaths;
use base64::Engine;
use chrono::Utc;
use std::fs;
use std::path::Path;

const MAX_NAME_CHARS: usize = 64;
const MAX_ID_CHARS: usize = 40;
const MIN_MEMORY_MB: u32 = 512;
const MAX_MEMORY_MB: u32 = 65_536;
const MAX_ICON_BYTES: u64 = 2 * 1024 * 1024;
const ICON_EXTENSIONS: [(&str, &str); 5] = [
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("webp", "image/webp"),
    ("gif", "image/gif"),
];

fn invalid(message: &str) -> AppError {
    AppError::InvalidInput(message.to_owned())
}

pub fn validate_id(id: &str) -> AppResult<()> {
    let valid = !id.is_empty()
        && id.len() <= MAX_ID_CHARS + 8
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    valid.then_some(()).ok_or_else(|| invalid("invalid instance id"))
}

fn clean_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS || name.chars().any(char::is_control) {
        return Err(invalid("the instance name must have 1-64 characters"));
    }
    Ok(name.to_owned())
}

fn validate_settings(settings: &InstanceSettings) -> AppResult<String> {
    if settings.game_version.trim().is_empty() {
        return Err(invalid("choose a game version"));
    }
    if !(MIN_MEMORY_MB..=MAX_MEMORY_MB).contains(&settings.memory_mb) {
        return Err(invalid("memory must be between 512 MB and 64 GB"));
    }
    if settings.java_path.as_deref().is_some_and(|path| path.trim().is_empty()) {
        return Err(invalid("the Java path cannot be empty"));
    }
    clean_name(&settings.name)
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    for character in name.to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug: String = slug.trim_matches('-').chars().take(MAX_ID_CHARS).collect();
    let slug = slug.trim_end_matches('-').to_owned();
    if slug.is_empty() {
        "instance".to_owned()
    } else {
        slug
    }
}

/// Creates the instance directory, picking a unique id derived from the name.
fn claim_directory(paths: &DataPaths, name: &str) -> AppResult<String> {
    fs::create_dir_all(paths.instances())?;
    let base = slugify(name);
    for attempt in 1.. {
        let id = if attempt == 1 { base.clone() } else { format!("{base}-{attempt}") };
        match fs::create_dir(paths.instance_dir(&id)) {
            Ok(()) => return Ok(id),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!("the attempt counter is unbounded")
}

fn save(paths: &DataPaths, instance: &Instance) -> AppResult<()> {
    let target = paths.instance_json(&instance.id);
    let temp = target.with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(instance)?)?;
    fs::rename(temp, target)?;
    Ok(())
}

pub fn get(paths: &DataPaths, id: &str) -> AppResult<Instance> {
    validate_id(id)?;
    let bytes = fs::read(paths.instance_json(id)).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => AppError::InstanceNotFound(id.to_owned()),
        _ => error.into(),
    })?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn list(paths: &DataPaths) -> AppResult<Vec<Instance>> {
    let Ok(entries) = fs::read_dir(paths.instances()) else {
        return Ok(Vec::new());
    };
    let mut instances: Vec<Instance> = entries
        .flatten()
        .filter_map(|entry| get(paths, &entry.file_name().to_string_lossy()).ok())
        .collect();
    instances.sort_by(|a, b| {
        b.last_played_at
            .cmp(&a.last_played_at)
            .then_with(|| b.created_at.cmp(&a.created_at))
    });
    Ok(instances)
}

pub fn create(paths: &DataPaths, new: NewInstance) -> AppResult<Instance> {
    let name = clean_name(&new.name)?;
    if new.game_version.trim().is_empty() {
        return Err(invalid("choose a game version"));
    }
    let id = claim_directory(paths, &name)?;

    let build = || -> AppResult<Instance> {
        fs::create_dir_all(paths.instance_game_dir(&id))?;
        let icon = apply_icon(paths, &id, new.icon.clone())?;
        let instance = Instance {
            schema_version: SCHEMA_VERSION,
            id: id.clone(),
            name,
            icon,
            game_version: new.game_version.clone(),
            loader: LoaderKind::Vanilla,
            memory_mb: DEFAULT_MEMORY_MB,
            jvm_args: Vec::new(),
            java_path: None,
            created_at: Utc::now(),
            last_played_at: None,
        };
        save(paths, &instance)?;
        Ok(instance)
    };
    build().inspect_err(|_| {
        let _ = fs::remove_dir_all(paths.instance_dir(&id));
    })
}

pub fn update(paths: &DataPaths, id: &str, settings: InstanceSettings) -> AppResult<Instance> {
    let mut instance = get(paths, id)?;
    instance.name = validate_settings(&settings)?;
    instance.game_version = settings.game_version.trim().to_owned();
    instance.memory_mb = settings.memory_mb;
    instance.jvm_args = settings.jvm_args;
    instance.java_path = settings.java_path.map(|path| path.trim().to_owned());
    save(paths, &instance)?;
    Ok(instance)
}

pub fn set_icon(paths: &DataPaths, id: &str, icon: IconInput) -> AppResult<Instance> {
    let mut instance = get(paths, id)?;
    instance.icon = apply_icon(paths, id, icon)?;
    save(paths, &instance)?;
    Ok(instance)
}

fn apply_icon(paths: &DataPaths, id: &str, icon: IconInput) -> AppResult<InstanceIcon> {
    match icon {
        IconInput::Preset { key } => {
            remove_custom_icons(paths, id);
            Ok(InstanceIcon::Preset { key })
        }
        IconInput::Custom { path } => {
            let source = Path::new(&path);
            let extension = source
                .extension()
                .map(|extension| extension.to_string_lossy().to_lowercase())
                .filter(|extension| ICON_EXTENSIONS.iter().any(|(known, _)| known == extension))
                .ok_or_else(|| invalid("the icon must be a PNG, JPG, WebP or GIF image"))?;
            if fs::metadata(source)?.len() > MAX_ICON_BYTES {
                return Err(invalid("the icon must be smaller than 2 MB"));
            }
            remove_custom_icons(paths, id);
            let file = format!("icon.{extension}");
            fs::copy(source, paths.instance_dir(id).join(&file))?;
            Ok(InstanceIcon::Custom { file })
        }
    }
}

fn remove_custom_icons(paths: &DataPaths, id: &str) {
    for (extension, _) in ICON_EXTENSIONS {
        let _ = fs::remove_file(paths.instance_dir(id).join(format!("icon.{extension}")));
    }
}

pub fn icon_data_url(paths: &DataPaths, id: &str) -> AppResult<Option<String>> {
    let InstanceIcon::Custom { file } = get(paths, id)?.icon else {
        return Ok(None);
    };
    let extension = Path::new(&file).extension().map(|extension| extension.to_string_lossy().to_lowercase());
    let mime = ICON_EXTENSIONS
        .iter()
        .find(|(known, _)| Some(*known) == extension.as_deref())
        .map(|(_, mime)| *mime)
        .ok_or_else(|| invalid("unsupported icon format"))?;
    let bytes = fs::read(paths.instance_dir(id).join(&file))?;
    Ok(Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )))
}

pub fn duplicate(paths: &DataPaths, id: &str) -> AppResult<Instance> {
    let source = get(paths, id)?;
    let name: String = format!("{} (copy)", source.name).chars().take(MAX_NAME_CHARS).collect();
    let new_id = claim_directory(paths, &name)?;

    let copy = || -> AppResult<Instance> {
        copy_directory(&paths.instance_dir(id), &paths.instance_dir(&new_id))?;
        let instance = Instance {
            id: new_id.clone(),
            name,
            created_at: Utc::now(),
            last_played_at: None,
            ..source
        };
        save(paths, &instance)?;
        Ok(instance)
    };
    copy().inspect_err(|_| {
        let _ = fs::remove_dir_all(paths.instance_dir(&new_id));
    })
}

fn copy_directory(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

pub fn delete(paths: &DataPaths, id: &str) -> AppResult<()> {
    get(paths, id)?;
    fs::remove_dir_all(paths.instance_dir(id))?;
    Ok(())
}

pub fn touch_last_played(paths: &DataPaths, id: &str) -> AppResult<()> {
    let mut instance = get(paths, id)?;
    instance.last_played_at = Some(Utc::now());
    save(paths, &instance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(name: &str) -> DataPaths {
        let root: PathBuf = std::env::temp_dir().join(format!("zenith-instances-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        DataPaths::new(root)
    }

    fn new(name: &str) -> NewInstance {
        NewInstance {
            name: name.into(),
            game_version: "1.21.1".into(),
            icon: IconInput::Preset { key: "cube".into() },
        }
    }

    fn settings(name: &str) -> InstanceSettings {
        InstanceSettings {
            name: name.into(),
            game_version: "1.20.4".into(),
            memory_mb: 4096,
            jvm_args: vec!["-XX:+UseG1GC".into()],
            java_path: Some("C:\\java\\bin\\java.exe".into()),
        }
    }

    #[test]
    fn creates_an_instance_with_a_unique_slug_and_game_directory() {
        let paths = scratch("create");

        let first = create(&paths, new("My Cool Pack!")).unwrap();
        let second = create(&paths, new("My Cool Pack!")).unwrap();

        assert_eq!(first.id, "my-cool-pack");
        assert_eq!(second.id, "my-cool-pack-2");
        assert!(paths.instance_game_dir(&first.id).is_dir());
        assert_eq!(get(&paths, &first.id).unwrap().name, "My Cool Pack!");
        let _ = fs::remove_dir_all(paths.instances());
    }

    #[test]
    fn rejects_bad_names_and_ids_that_could_escape_the_instances_directory() {
        let paths = scratch("validation");

        assert!(matches!(create(&paths, new("   ")), Err(AppError::InvalidInput(_))));
        assert!(matches!(create(&paths, new(&"x".repeat(65))), Err(AppError::InvalidInput(_))));
        for id in ["../secrets", "a/b", "A", "", "a\\b"] {
            assert!(matches!(get(&paths, id), Err(AppError::InvalidInput(_))), "{id}");
        }
        assert!(matches!(get(&paths, "missing"), Err(AppError::InstanceNotFound(_))));
    }

    #[test]
    fn updates_settings_and_validates_memory() {
        let paths = scratch("update");
        let instance = create(&paths, new("Pack")).unwrap();

        let updated = update(&paths, &instance.id, settings("Renamed")).unwrap();

        assert_eq!((updated.name.as_str(), updated.memory_mb), ("Renamed", 4096));
        assert_eq!(updated.id, instance.id);
        assert_eq!(get(&paths, &instance.id).unwrap().jvm_args, ["-XX:+UseG1GC"]);

        let too_small = InstanceSettings { memory_mb: 64, ..settings("Renamed") };
        assert!(matches!(update(&paths, &instance.id, too_small), Err(AppError::InvalidInput(_))));
        let _ = fs::remove_dir_all(paths.instances());
    }

    #[test]
    fn duplicates_an_instance_with_its_files_and_a_fresh_identity() {
        let paths = scratch("duplicate");
        let original = create(&paths, new("Survival")).unwrap();
        let saves = paths.instance_game_dir(&original.id).join("saves").join("World1");
        fs::create_dir_all(&saves).unwrap();
        fs::write(saves.join("level.dat"), b"world").unwrap();
        touch_last_played(&paths, &original.id).unwrap();

        let copy = duplicate(&paths, &original.id).unwrap();

        assert_eq!(copy.name, "Survival (copy)");
        assert_ne!(copy.id, original.id);
        assert!(copy.last_played_at.is_none());
        let copied = paths.instance_game_dir(&copy.id).join("saves").join("World1").join("level.dat");
        assert_eq!(fs::read(copied).unwrap(), b"world");
        assert_eq!(get(&paths, &original.id).unwrap().name, "Survival");
        let _ = fs::remove_dir_all(paths.instances());
    }

    #[test]
    fn deletes_an_instance_and_all_its_files() {
        let paths = scratch("delete");
        let instance = create(&paths, new("Temp")).unwrap();

        delete(&paths, &instance.id).unwrap();

        assert!(!paths.instance_dir(&instance.id).exists());
        assert!(matches!(delete(&paths, &instance.id), Err(AppError::InstanceNotFound(_))));
        let _ = fs::remove_dir_all(paths.instances());
    }

    #[test]
    fn lists_recently_played_first_and_skips_corrupted_instances() {
        let paths = scratch("list");
        let older = create(&paths, new("Older")).unwrap();
        let newer = create(&paths, new("Newer")).unwrap();
        touch_last_played(&paths, &older.id).unwrap();
        fs::create_dir_all(paths.instance_dir("broken")).unwrap();
        fs::write(paths.instance_json("broken"), b"{ not json").unwrap();

        let ids: Vec<String> = list(&paths).unwrap().into_iter().map(|instance| instance.id).collect();

        assert_eq!(ids, [older.id, newer.id]);
        let _ = fs::remove_dir_all(paths.instances());
    }

    #[test]
    fn stores_a_custom_icon_and_serves_it_as_a_data_url() {
        let paths = scratch("icon");
        fs::create_dir_all(paths.instances()).unwrap();
        let image = paths.instances().join("source.png");
        fs::write(&image, [0x89, b'P', b'N', b'G']).unwrap();

        let instance = create(
            &paths,
            NewInstance { icon: IconInput::Custom { path: image.to_string_lossy().into() }, ..new("Iconic") },
        )
        .unwrap();

        assert_eq!(instance.icon, InstanceIcon::Custom { file: "icon.png".into() });
        let url = icon_data_url(&paths, &instance.id).unwrap().unwrap();
        assert!(url.starts_with("data:image/png;base64,"));

        let reset = set_icon(&paths, &instance.id, IconInput::Preset { key: "sword".into() }).unwrap();
        assert!(matches!(reset.icon, InstanceIcon::Preset { .. }));
        assert!(!paths.instance_dir(&instance.id).join("icon.png").exists());
        assert_eq!(icon_data_url(&paths, &instance.id).unwrap(), None);

        let text = paths.instances().join("notes.txt");
        fs::write(&text, "x").unwrap();
        let rejected = set_icon(&paths, &instance.id, IconInput::Custom { path: text.to_string_lossy().into() });
        assert!(matches!(rejected, Err(AppError::InvalidInput(_))));
        let _ = fs::remove_dir_all(paths.instances());
    }

    #[test]
    fn failed_creation_leaves_no_directory_behind() {
        let paths = scratch("rollback");
        let bad_icon = IconInput::Custom { path: "missing.png".into() };

        let result = create(&paths, NewInstance { icon: bad_icon, ..new("Ghost") });

        assert!(result.is_err());
        assert!(!paths.instance_dir("ghost").exists());
    }
}
