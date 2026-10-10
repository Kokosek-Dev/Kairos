use crate::error::{AppError, AppResult};
use crate::http::get_json;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionKind {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: VersionKind,
    pub url: String,
    pub sha1: String,
    pub release_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<ManifestEntry>,
}

pub async fn fetch_manifest(client: &Client) -> AppResult<VersionManifest> {
    get_json(client, MANIFEST_URL).await
}

impl VersionManifest {
    pub fn find(&self, id: &str) -> AppResult<&ManifestEntry> {
        self.versions
            .iter()
            .find(|entry| entry.id == id)
            .ok_or_else(|| AppError::VersionNotFound(id.to_owned()))
    }
}
