use super::LoaderVersion;
use crate::error::{AppError, AppResult};
use crate::mojang::profile::is_safe_id;
use crate::mojang::version::RawVersion;
use crate::paths::DataPaths;
use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;
use tokio::fs;

/// Fabric and Quilt publish the same "meta" API shape, so one implementation serves both.
#[derive(Clone, Copy)]
pub struct MetaApi {
    base: &'static str,
}

pub const FABRIC: MetaApi = MetaApi { base: "https://meta.fabricmc.net/v2/versions/loader" };
pub const QUILT: MetaApi = MetaApi { base: "https://meta.quiltmc.org/v3/versions/loader" };

#[derive(Debug, Deserialize)]
struct LoaderEntry {
    loader: LoaderInfo,
}

#[derive(Debug, Deserialize)]
struct LoaderInfo {
    version: String,
    stable: Option<bool>,
}

fn url(api: MetaApi, segments: &[&str]) -> AppResult<Url> {
    let mut url = Url::parse(api.base).map_err(|error| AppError::Internal(error.to_string()))?;
    url.path_segments_mut()
        .map_err(|()| AppError::Internal("invalid meta URL".into()))?
        .extend(segments);
    Ok(url)
}

pub async fn list_versions(client: &Client, api: MetaApi, game_version: &str) -> AppResult<Vec<LoaderVersion>> {
    let response = client.get(url(api, &[game_version])?).send().await?;
    if matches!(response.status(), StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND) {
        return Ok(Vec::new());
    }
    let entries: Vec<LoaderEntry> = response.error_for_status()?.json().await?;

    let mut recommended_taken = false;
    Ok(entries
        .into_iter()
        .map(|entry| {
            let stable = entry.loader.stable.unwrap_or_else(|| !entry.loader.version.contains('-'));
            let recommended = stable && !recommended_taken;
            recommended_taken |= recommended;
            LoaderVersion { version: entry.loader.version, recommended }
        })
        .collect())
}

/// Downloads the launcher profile for a loader/game version pair and stores it as a version.
pub async fn install(
    client: &Client,
    paths: &DataPaths,
    api: MetaApi,
    game_version: &str,
    loader_version: &str,
) -> AppResult<String> {
    let bytes = client
        .get(url(api, &[game_version, loader_version, "profile", "json"])?)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let profile: RawVersion = serde_json::from_slice(&bytes)?;
    if !is_safe_id(&profile.id) || profile.inherits_from.as_deref() != Some(game_version) {
        return Err(AppError::Internal("the loader profile does not match the game version".into()));
    }

    fs::create_dir_all(paths.version_dir(&profile.id)).await?;
    let target = paths.version_json(&profile.id);
    let temp = target.with_extension("json.tmp");
    fs::write(&temp, &bytes).await?;
    fs::rename(&temp, &target).await?;
    Ok(profile.id)
}
