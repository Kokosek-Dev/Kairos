mod installer;
mod profile;

use crate::error::{AppError, AppResult};
use crate::instances::model::LoaderKind;
use crate::mojang::profile::is_safe_id;
use crate::paths::DataPaths;
use reqwest::Client;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersion {
    pub version: String,
    pub recommended: bool,
}

pub async fn list_versions(client: &Client, kind: LoaderKind, game_version: &str) -> AppResult<Vec<LoaderVersion>> {
    match kind {
        LoaderKind::Vanilla => Ok(Vec::new()),
        LoaderKind::Fabric => profile::list_versions(client, profile::FABRIC, game_version).await,
        LoaderKind::Quilt => profile::list_versions(client, profile::QUILT, game_version).await,
        LoaderKind::Forge => installer::forge_versions(client, game_version).await,
        LoaderKind::NeoForge => installer::neoforge_versions(client, game_version).await,
    }
}

pub struct InstallContext<'a> {
    pub client: &'a Client,
    pub paths: &'a DataPaths,
    pub java: &'a Path,
    pub cancel: &'a CancellationToken,
}

fn kind_label(kind: LoaderKind) -> &'static str {
    match kind {
        LoaderKind::Vanilla => "vanilla",
        LoaderKind::Fabric => "fabric",
        LoaderKind::Quilt => "quilt",
        LoaderKind::Forge => "forge",
        LoaderKind::NeoForge => "neoforge",
    }
}

fn file_safe(text: &str) -> String {
    text.chars()
        .map(|character| if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') { character } else { '_' })
        .collect()
}

/// Remembers which version profile a loader/game pair installed, because installers
/// choose their own profile names.
fn record_path(paths: &DataPaths, kind: LoaderKind, game_version: &str, loader_version: &str) -> PathBuf {
    paths.loader_records().join(format!(
        "{}-{}-{}.id",
        kind_label(kind),
        file_safe(game_version),
        file_safe(loader_version)
    ))
}

/// Makes sure the loader is installed and returns the id of the version profile to launch.
pub async fn ensure_installed(
    context: &InstallContext<'_>,
    kind: LoaderKind,
    game_version: &str,
    loader_version: &str,
    on_output: &(impl Fn(String) + Sync),
) -> AppResult<String> {
    let paths = context.paths;
    let record = record_path(paths, kind, game_version, loader_version);
    if let Ok(id) = fs::read_to_string(&record).await {
        let id = id.trim().to_owned();
        if is_safe_id(&id) && fs::try_exists(paths.version_json(&id)).await? {
            return Ok(id);
        }
    }

    let id = match kind {
        LoaderKind::Vanilla => return Ok(game_version.to_owned()),
        LoaderKind::Fabric => profile::install(context.client, paths, profile::FABRIC, game_version, loader_version).await?,
        LoaderKind::Quilt => profile::install(context.client, paths, profile::QUILT, game_version, loader_version).await?,
        LoaderKind::Forge | LoaderKind::NeoForge => {
            let spec = if kind == LoaderKind::Forge {
                installer::forge_installer(game_version, loader_version)
            } else {
                installer::neoforge_installer(game_version, loader_version)
            };
            installer::install(context.client, paths, &spec, context.java, context.cancel, on_output).await?
        }
    };

    fs::create_dir_all(paths.loader_records()).await?;
    fs::write(&record, &id).await?;
    Ok(id)
}

pub fn require_version(kind: LoaderKind, loader_version: Option<&str>) -> AppResult<&str> {
    loader_version
        .filter(|version| !version.is_empty())
        .ok_or_else(|| AppError::InvalidInput(format!("the {} instance has no loader version", kind_label(kind))))
}
