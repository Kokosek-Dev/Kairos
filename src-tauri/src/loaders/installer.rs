use super::LoaderVersion;
use crate::download::{download_all, Checksum, Counters, DownloadItem};
use crate::error::{AppError, AppResult};
use crate::java::hide_console;
use crate::mojang::profile::is_safe_id;
use crate::paths::DataPaths;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

const FORGE_MAVEN: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";
const FORGE_PROMOTIONS: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
const NEOFORGE_MAVEN: &str = "https://maven.neoforged.net/releases/net/neoforged/neoforge";
const NEOFORGE_VERSIONS: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
const LAUNCHER_PROFILES: &str = r#"{"profiles":{},"version":3}"#;

/// Forge and NeoForge ship an installer that patches the game; it is run in its own
/// headless "install client" mode against Kairos's data directory, which has the same
/// layout as a `.minecraft` folder.
pub struct Installer {
    pub url: String,
    pub file_name: String,
    pub game_version: String,
}

pub fn forge_installer(game_version: &str, loader_version: &str) -> Installer {
    let full = format!("{game_version}-{loader_version}");
    Installer {
        url: format!("{FORGE_MAVEN}/{full}/forge-{full}-installer.jar"),
        file_name: format!("forge-{full}-installer.jar"),
        game_version: game_version.to_owned(),
    }
}

pub fn neoforge_installer(game_version: &str, loader_version: &str) -> Installer {
    Installer {
        url: format!("{NEOFORGE_MAVEN}/{loader_version}/neoforge-{loader_version}-installer.jar"),
        file_name: format!("neoforge-{loader_version}-installer.jar"),
        game_version: game_version.to_owned(),
    }
}

pub fn forge_versions_from_metadata(metadata: &str, game_version: &str) -> Vec<String> {
    let prefix = format!("{game_version}-");
    metadata
        .split("<version>")
        .skip(1)
        .filter_map(|chunk| chunk.split("</version>").next())
        .filter_map(|full| full.strip_prefix(&prefix))
        .map(str::to_owned)
        .collect()
}

pub async fn forge_versions(client: &Client, game_version: &str) -> AppResult<Vec<LoaderVersion>> {
    let metadata = client
        .get(format!("{FORGE_MAVEN}/maven-metadata.xml"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let mut versions = forge_versions_from_metadata(&metadata, game_version);
    versions.reverse();
    if versions.is_empty() {
        return Ok(Vec::new());
    }

    let promotions: Value = client.get(FORGE_PROMOTIONS).send().await?.error_for_status()?.json().await?;
    let promo = |kind: &str| {
        promotions["promos"][format!("{game_version}-{kind}")]
            .as_str()
            .map(str::to_owned)
    };
    let recommended = promo("recommended").or_else(|| promo("latest")).unwrap_or_else(|| versions[0].clone());

    Ok(versions
        .into_iter()
        .map(|version| LoaderVersion { recommended: version == recommended, version })
        .collect())
}

/// NeoForge versions encode the game version: `21.1.258` is for 1.21.1, `21.0.x` for 1.21 and
/// `26.3.0.67-beta` for 26.3. The result is double-checked against the installer when it is used.
pub fn neoforge_game_version(version: &str) -> Option<String> {
    let numbers: Vec<u32> = version
        .split(['.', '-'])
        .map_while(|part| part.parse().ok())
        .collect();
    let (major, minor, patch) = (*numbers.first()?, *numbers.get(1)?, numbers.get(2).copied().unwrap_or(0));
    if major < 26 {
        return Some(if minor == 0 { format!("1.{major}") } else { format!("1.{major}.{minor}") });
    }
    Some(if patch == 0 { format!("{major}.{minor}") } else { format!("{major}.{minor}.{patch}") })
}

pub async fn neoforge_versions(client: &Client, game_version: &str) -> AppResult<Vec<LoaderVersion>> {
    let listing: HashMap<String, Value> = client
        .get(NEOFORGE_VERSIONS)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let all: Vec<&str> = listing
        .get("versions")
        .and_then(Value::as_array)
        .map(|versions| versions.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    let mut matching: Vec<&str> = all
        .into_iter()
        .filter(|version| neoforge_game_version(version).as_deref() == Some(game_version))
        .collect();
    matching.reverse();
    let recommended = matching
        .iter()
        .find(|version| !version.contains('-'))
        .or(matching.first())
        .copied()
        .map(str::to_owned);

    Ok(matching
        .into_iter()
        .map(|version| LoaderVersion { recommended: Some(version) == recommended.as_deref(), version: version.to_owned() })
        .collect())
}

struct JarInfo {
    version_id: String,
    game_version: String,
}

/// Reads which version profile the installer will create and which game it patches.
fn read_jar_info(jar: &Path) -> AppResult<JarInfo> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(jar)?).map_err(|error| AppError::Internal(error.to_string()))?;
    let mut read = |name: &str| -> Option<Value> {
        let entry = archive.by_name(name).ok()?;
        serde_json::from_reader(entry).ok()
    };
    let profile = read("install_profile.json").ok_or_else(|| AppError::Internal("the installer has no install profile".into()))?;
    let modern_version = read("version.json");

    let version_id = modern_version
        .as_ref()
        .and_then(|version| version["id"].as_str())
        .or_else(|| profile["versionInfo"]["id"].as_str())
        .ok_or_else(|| AppError::Internal("the installer does not name its version".into()))?
        .to_owned();
    let game_version = profile["minecraft"]
        .as_str()
        .or_else(|| profile["install"]["minecraft"].as_str())
        .ok_or_else(|| AppError::Internal("the installer does not name its game version".into()))?
        .to_owned();
    Ok(JarInfo { version_id, game_version })
}

async fn fetch_sha256(client: &Client, installer_url: &str) -> AppResult<String> {
    let text = client
        .get(format!("{installer_url}.sha256"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    text.split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| AppError::Internal("the installer checksum is empty".into()))
}

async fn next_line<R: AsyncRead + Unpin>(reader: &mut BufReader<R>) -> Option<String> {
    let mut buffer = Vec::new();
    match reader.read_until(b'\n', &mut buffer).await {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(String::from_utf8_lossy(&buffer).trim_end_matches(['\r', '\n']).to_owned()),
    }
}

pub async fn install(
    client: &Client,
    paths: &DataPaths,
    installer: &Installer,
    java: &Path,
    cancel: &CancellationToken,
    on_output: &(impl Fn(String) + Sync),
) -> AppResult<String> {
    let jar = paths.installers().join(&installer.file_name);
    let checksum = fetch_sha256(client, &installer.url).await?;
    let item = DownloadItem {
        url: installer.url.clone(),
        path: jar.clone(),
        checksum: Some(Checksum::Sha256(checksum)),
        size: None,
    };
    download_all(client, vec![item], cancel, &Arc::new(Counters::default())).await?;

    let jar_for_task = jar.clone();
    let info = tokio::task::spawn_blocking(move || read_jar_info(&jar_for_task))
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??;
    if info.game_version != installer.game_version {
        return Err(AppError::InvalidInput(format!(
            "this loader version is for Minecraft {}, not {}",
            info.game_version, installer.game_version
        )));
    }
    if !is_safe_id(&info.version_id) {
        return Err(AppError::Internal("the installer names an invalid version".into()));
    }

    fs::create_dir_all(paths.root()).await?;
    if !fs::try_exists(paths.launcher_profiles()).await? {
        fs::write(paths.launcher_profiles(), LAUNCHER_PROFILES).await?;
    }

    let mut command = Command::new(java);
    command
        .arg("-jar")
        .arg(&jar)
        .arg("--installClient")
        .arg(paths.root())
        .current_dir(paths.root())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    hide_console(&mut command);
    let mut child = command.spawn()?;
    let mut stdout = child.stdout.take().map(BufReader::new);
    let mut stderr = child.stderr.take().map(BufReader::new);

    while stdout.is_some() || stderr.is_some() {
        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = child.kill().await;
                return Err(AppError::Cancelled);
            }
            line = async { next_line(stdout.as_mut().unwrap()).await }, if stdout.is_some() => match line {
                Some(line) => on_output(line),
                None => stdout = None,
            },
            line = async { next_line(stderr.as_mut().unwrap()).await }, if stderr.is_some() => match line {
                Some(line) => on_output(line),
                None => stderr = None,
            },
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(AppError::Internal(format!(
            "the loader installer failed (exit code {})",
            status.code().map_or_else(|| "unknown".to_owned(), |code| code.to_string())
        )));
    }
    if !fs::try_exists(paths.version_json(&info.version_id)).await? {
        return Err(AppError::Internal("the loader installer did not create its version".into()));
    }
    Ok(info.version_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_neoforge_versions_to_game_versions() {
        let cases = [
            ("21.1.258", "1.21.1"),
            ("21.0.167", "1.21"),
            ("20.4.237", "1.20.4"),
            ("20.2.3-beta", "1.20.2"),
            ("26.3.0.67-beta", "26.3"),
            ("26.1.2.5", "26.1.2"),
        ];
        for (neoforge, game) in cases {
            assert_eq!(neoforge_game_version(neoforge).as_deref(), Some(game), "{neoforge}");
        }
        assert_eq!(neoforge_game_version("garbage"), None);
    }

    #[test]
    fn lists_forge_versions_for_exactly_one_game_version() {
        let metadata = "<versions><version>1.20.1-47.4.25</version><version>1.20.1-47.4.26</version>\
            <version>1.20-46.0.14</version><version>1.7.10-10.13.4.1614-1.7.10</version><version>1.20.10-1.0</version></versions>";

        assert_eq!(forge_versions_from_metadata(metadata, "1.20.1"), ["47.4.25", "47.4.26"]);
        assert_eq!(forge_versions_from_metadata(metadata, "1.7.10"), ["10.13.4.1614-1.7.10"]);
        assert!(forge_versions_from_metadata(metadata, "1.19").is_empty());
    }

    #[test]
    fn builds_installer_urls_from_maven_coordinates() {
        let forge = forge_installer("1.20.1", "47.4.26");
        assert_eq!(
            forge.url,
            "https://maven.minecraftforge.net/net/minecraftforge/forge/1.20.1-47.4.26/forge-1.20.1-47.4.26-installer.jar"
        );
        let neoforge = neoforge_installer("1.21.1", "21.1.258");
        assert!(neoforge.url.ends_with("/21.1.258/neoforge-21.1.258-installer.jar"));
    }
}
