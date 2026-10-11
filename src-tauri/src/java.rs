use crate::download::{download_with_progress, Checksum, DownloadItem};
use crate::error::{AppError, AppResult};
use crate::http::get_json;
use crate::mojang::platform::os_name;
use crate::paths::DataPaths;
use reqwest::Client;
use serde::Deserialize;
use std::path::{Component, Path, PathBuf};
use tokio::fs;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

const ADOPTIUM_API: &str = "https://api.adoptium.net/v3/assets/latest";

#[derive(Debug, Deserialize)]
struct AdoptiumAsset {
    binary: AdoptiumBinary,
}

#[derive(Debug, Deserialize)]
struct AdoptiumBinary {
    package: AdoptiumPackage,
}

#[derive(Debug, Deserialize)]
struct AdoptiumPackage {
    link: String,
    name: String,
    size: u64,
    checksum: String,
}

fn executable_name() -> &'static str {
    if cfg!(windows) {
        "java.exe"
    } else {
        "java"
    }
}

fn java_in_home(home: &Path) -> Option<PathBuf> {
    [home.join("bin"), home.join("Contents").join("Home").join("bin")]
        .into_iter()
        .map(|bin| bin.join(executable_name()))
        .find(|candidate| candidate.is_file())
}

fn adoptium_os() -> &'static str {
    match os_name() {
        "osx" => "mac",
        other => other,
    }
}

fn adoptium_arch() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "x64"
    }
}

pub fn parse_major(version_output: &str) -> Option<u32> {
    let quoted = version_output.split('"').nth(1)?;
    let mut parts = quoted.split(['.', '-', '+', '_']);
    let first: u32 = parts.next()?.parse().ok()?;
    if first == 1 {
        parts.next()?.parse().ok()
    } else {
        Some(first)
    }
}

async fn probe_major(java: &Path) -> Option<u32> {
    let mut command = Command::new(java);
    command.arg("-version");
    hide_console(&mut command);
    let output = command.output().await.ok()?;
    parse_major(&String::from_utf8_lossy(&output.stderr))
}

#[cfg(windows)]
pub fn hide_console(command: &mut Command) {
    command.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
pub fn hide_console(_command: &mut Command) {}

pub async fn find(paths: &DataPaths, major: u32) -> Option<PathBuf> {
    if let Some(managed) = java_in_home(&paths.runtimes().join(major.to_string())) {
        return Some(managed);
    }

    let mut candidates = Vec::new();
    if let Some(home) = std::env::var_os("JAVA_HOME") {
        candidates.extend(java_in_home(Path::new(&home)));
    }
    candidates.push(PathBuf::from(executable_name()));

    for candidate in candidates {
        if probe_major(&candidate).await == Some(major) {
            return Some(candidate);
        }
    }
    None
}

pub async fn ensure(
    client: &Client,
    paths: &DataPaths,
    major: u32,
    cancel: &CancellationToken,
    on_progress: impl FnMut(u64, u64),
) -> AppResult<PathBuf> {
    if let Some(java) = find(paths, major).await {
        return Ok(java);
    }
    install_managed(client, paths, major, cancel, on_progress).await
}

async fn install_managed(
    client: &Client,
    paths: &DataPaths,
    major: u32,
    cancel: &CancellationToken,
    mut on_progress: impl FnMut(u64, u64),
) -> AppResult<PathBuf> {
    let url = format!(
        "{ADOPTIUM_API}/{major}/hotspot?architecture={}&image_type=jre&os={}&vendor=eclipse",
        adoptium_arch(),
        adoptium_os()
    );
    let assets: Vec<AdoptiumAsset> = get_json(client, &url).await?;
    let package = assets
        .into_iter()
        .next()
        .ok_or(AppError::JavaUnavailable(major))?
        .binary
        .package;

    let archive = paths.runtimes().join("downloads").join(&package.name);
    let item = DownloadItem {
        url: package.link,
        path: archive.clone(),
        checksum: Some(Checksum::Sha256(package.checksum)),
        size: Some(package.size),
    };
    let total = package.size;
    download_with_progress(client, vec![item], cancel, |counters| on_progress(counters.bytes(), total)).await?;

    let target = paths.runtimes().join(major.to_string());
    let staging = paths.runtimes().join(format!("{major}.partial"));
    let (archive_for_task, staging_for_task) = (archive.clone(), staging.clone());
    tokio::task::spawn_blocking(move || extract_archive(&archive_for_task, &staging_for_task))
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??;

    if fs::try_exists(&target).await? {
        fs::remove_dir_all(&target).await?;
    }
    fs::rename(&staging, &target).await?;
    let _ = fs::remove_file(&archive).await;

    java_in_home(&target).ok_or(AppError::JavaUnavailable(major))
}

fn relative_without_top_level(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    components.next()?;
    let relative: PathBuf = components.collect();
    let safe = relative.components().all(|part| matches!(part, Component::Normal(_)));
    (safe && relative.components().next().is_some()).then_some(relative)
}

fn extract_archive(archive: &Path, destination: &Path) -> AppResult<()> {
    if destination.exists() {
        std::fs::remove_dir_all(destination)?;
    }
    std::fs::create_dir_all(destination)?;
    let is_zip = archive.extension().is_some_and(|extension| extension == "zip");
    if is_zip {
        extract_zip(archive, destination)
    } else {
        extract_tar_gz(archive, destination)
    }
}

fn extract_zip(archive: &Path, destination: &Path) -> AppResult<()> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(archive)?)
        .map_err(|error| AppError::Internal(error.to_string()))?;
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .map_err(|error| AppError::Internal(error.to_string()))?;
        let Some(relative) = entry.enclosed_name().and_then(|name| relative_without_top_level(&name)) else {
            continue;
        };
        let target = destination.join(relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::io::copy(&mut entry, &mut std::fs::File::create(&target)?)?;
        }
    }
    Ok(())
}

fn extract_tar_gz(archive: &Path, destination: &Path) -> AppResult<()> {
    let decoder = flate2::read::GzDecoder::new(std::fs::File::open(archive)?);
    for entry in tar::Archive::new(decoder).entries()? {
        let mut entry = entry?;
        let Some(relative) = relative_without_top_level(&entry.path()?) else {
            continue;
        };
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        entry.unpack(&target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_modern_and_legacy_version_strings() {
        assert_eq!(parse_major("openjdk version \"21.0.4\" 2024-07-16"), Some(21));
        assert_eq!(parse_major("openjdk version \"25\" 2025-09-16"), Some(25));
        assert_eq!(parse_major("java version \"1.8.0_301\""), Some(8));
        assert_eq!(parse_major("openjdk version \"17.0.20.1\" 2026-01-20"), Some(17));
        assert_eq!(parse_major("command not found"), None);
    }

    #[test]
    fn extracts_zip_and_strips_the_top_level_directory() {
        let dir = std::env::temp_dir().join(format!("kairos-java-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let archive = dir.join("runtime.zip");

        let mut writer = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        writer.add_directory("jdk-21/", options).unwrap();
        writer.start_file("jdk-21/bin/java.exe", options).unwrap();
        writer.write_all(b"binary").unwrap();
        writer.start_file("../escape.txt", options).unwrap();
        writer.write_all(b"nope").unwrap();
        writer.finish().unwrap();

        let target = dir.join("out");
        extract_archive(&archive, &target).unwrap();

        assert_eq!(std::fs::read(target.join("bin").join("java.exe")).unwrap(), b"binary");
        assert!(!dir.join("escape.txt").exists());
        assert!(!target.join("escape.txt").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
