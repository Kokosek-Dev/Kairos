use crate::error::{AppError, AppResult};
use crate::instances::model::Instance;
use crate::instances::store;
use crate::paths::DataPaths;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

const SECRET_FLAGS: [&str; 5] = ["--accessToken", "--xuid", "--clientId", "--uuid", "--username"];
const LOG_TAIL_LINES: usize = 1000;
const MAX_FILE_BYTES: usize = 400 * 1024;

fn zip_error(error: zip::result::ZipError) -> AppError {
    AppError::Internal(error.to_string())
}

/// Hides what must never leave the computer: the values of sign-in related arguments and the user's home folder.
pub fn redact(text: &str, home: Option<&str>) -> String {
    let mut result = text.to_owned();
    if let Some(home) = home.filter(|home| home.len() > 3) {
        result = result.replace(home, "~").replace(&home.replace('\\', "/"), "~");
    }
    for flag in SECRET_FLAGS {
        let mut hidden = String::with_capacity(result.len());
        let mut rest = result.as_str();
        while let Some(position) = rest.find(flag) {
            let after = position + flag.len();
            hidden.push_str(&rest[..after]);
            let tail = &rest[after..];
            let value_start = tail.len() - tail.trim_start_matches([' ', '=']).len();
            hidden.push_str(&tail[..value_start]);
            let value = &tail[value_start..];
            let value_end = value.find(|character: char| character.is_whitespace() || character == '"').unwrap_or(value.len());
            if value_end > 0 {
                hidden.push_str("***");
            }
            rest = &value[value_end..];
        }
        hidden.push_str(rest);
        result = hidden;
    }
    result
}

fn newest(folder: &Path, prefix: &str, suffix: &str) -> Option<PathBuf> {
    fs::read_dir(folder)
        .ok()?
        .flatten()
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with(prefix) && name.ends_with(suffix)
        })
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn read_text(path: &Path, tail_lines: Option<usize>) -> String {
    let bytes = fs::read(path).unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_FILE_BYTES)]).into_owned();
    match tail_lines {
        Some(count) => {
            let lines: Vec<&str> = text.lines().collect();
            lines[lines.len().saturating_sub(count)..].join("\n")
        }
        None => text,
    }
}

fn mod_names(game_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(game_dir.join("mods"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".jar") || name.ends_with(".jar.disabled"))
        .collect();
    names.sort();
    names
}

fn summary(paths: &DataPaths, instances: &[Instance], selected: Option<&Instance>, version: &str, total_mb: u64) -> String {
    let mut text = format!(
        "Kairos {version}\nSystem: {} {} ({}), {total_mb} MB memory\nData folder: {}\n\nInstances:\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        sysinfo::System::long_os_version().unwrap_or_default(),
        paths.root().display()
    );
    for instance in instances {
        let loader = instance.loader_version.as_deref().map(|loader| format!(" {loader}")).unwrap_or_default();
        text.push_str(&format!(
            "- {} | Minecraft {} | {:?}{loader} | {} MB | played {} min\n",
            instance.name,
            instance.game_version,
            instance.loader,
            instance.memory_mb,
            instance.play_time_secs / 60
        ));
    }
    if let Some(instance) = selected {
        text.push_str(&format!("\nSelected instance: {}\nJVM arguments: {}\nMods:\n", instance.name, instance.jvm_args.join(" ")));
        for name in mod_names(&paths.instance_game_dir(&instance.id)) {
            text.push_str(&format!("- {name}\n"));
        }
    }
    text
}

fn add(zip: &mut zip::ZipWriter<File>, name: &str, text: &str) -> AppResult<()> {
    zip.start_file(name, SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)).map_err(zip_error)?;
    zip.write_all(text.as_bytes())?;
    Ok(())
}

/// Writes a zip with what is needed to help with a problem: system and instance overview, mod names, the latest log and crash reports.
pub fn write_report(paths: &DataPaths, instance_id: Option<&str>, destination: &Path, version: &str, total_mb: u64) -> AppResult<()> {
    let instances = store::list(paths)?;
    let selected = match instance_id {
        Some(id) => instances.iter().find(|instance| instance.id == id),
        None => instances.iter().max_by_key(|instance| instance.last_played_at),
    };
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok();
    let hide = |text: &str| redact(text, home.as_deref());

    let mut zip = zip::ZipWriter::new(File::create(destination)?);
    add(&mut zip, "report.txt", &hide(&summary(paths, &instances, selected, version, total_mb)))?;
    if let Some(instance) = selected {
        let game_dir = paths.instance_game_dir(&instance.id);
        let log = game_dir.join("logs").join("latest.log");
        if log.is_file() {
            add(&mut zip, "latest.log", &hide(&read_text(&log, Some(LOG_TAIL_LINES))))?;
        }
        if let Some(path) = newest(&game_dir.join("crash-reports"), "crash-", ".txt") {
            add(&mut zip, "crash-report.txt", &hide(&read_text(&path, None)))?;
        }
        if let Some(path) = newest(&game_dir, "hs_err_pid", ".log") {
            add(&mut zip, "java-crash.log", &hide(&read_text(&path, None)))?;
        }
    }
    zip.finish().map_err(zip_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::model::{IconInput, LoaderKind, NewInstance};
    use std::io::Read;

    #[test]
    fn hides_sign_in_values_and_the_home_folder() {
        let line = "Command Line: java -cp C:\\Users\\filip\\lib.jar Main --username Steve --uuid 1234 --accessToken eyJhbGciOi.abc --xuid=99 --version 1.21";
        let hidden = redact(line, Some("C:\\Users\\filip"));
        assert!(!hidden.contains("eyJhbGciOi") && !hidden.contains("1234") && !hidden.contains("Steve") && !hidden.contains("filip") && !hidden.contains("=99"));
        assert!(hidden.contains("--accessToken ***") && hidden.contains("--xuid=***") && hidden.contains("--version 1.21"));
        assert_eq!(redact("nothing secret", None), "nothing secret");
        assert_eq!(redact("--accessToken", None), "--accessToken");
    }

    #[test]
    fn bundles_the_overview_logs_and_crash_reports_without_secrets() {
        let paths = DataPaths::new(std::env::temp_dir().join(format!("kairos-diag-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        let new = NewInstance {
            name: "Report me".into(),
            game_version: "1.21".into(),
            icon: IconInput::Preset { key: "cube".into() },
            loader: LoaderKind::Vanilla,
            loader_version: None,
        };
        let instance = store::create(&paths, new).unwrap();
        let game = paths.instance_game_dir(&instance.id);
        fs::create_dir_all(game.join("logs")).unwrap();
        fs::create_dir_all(game.join("mods")).unwrap();
        fs::create_dir_all(game.join("crash-reports")).unwrap();
        fs::write(game.join("logs/latest.log"), "starting\nlaunch --accessToken SECRETTOKEN done").unwrap();
        fs::write(game.join("mods/example.jar"), b"x").unwrap();
        fs::write(game.join("crash-reports/crash-1.txt"), "Description: boom").unwrap();

        let target = paths.root().join("report.zip");
        write_report(&paths, None, &target, "9.9.9", 16384).unwrap();

        let mut zip = zip::ZipArchive::new(File::open(&target).unwrap()).unwrap();
        let mut read = |name: &str| {
            let mut text = String::new();
            zip.by_name(name).unwrap().read_to_string(&mut text).unwrap();
            text
        };
        let overview = read("report.txt");
        assert!(overview.contains("Kairos 9.9.9") && overview.contains("Report me") && overview.contains("example.jar"));
        let log = read("latest.log");
        assert!(log.contains("--accessToken ***") && !log.contains("SECRETTOKEN"));
        assert_eq!(read("crash-report.txt"), "Description: boom");
        let _ = fs::remove_dir_all(paths.root());
    }
}
