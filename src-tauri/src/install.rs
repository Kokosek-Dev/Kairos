use crate::download::{download_all, download_with_progress, sha1_bytes, Checksum, DownloadItem};
use crate::error::{AppError, AppResult};
use crate::mojang::assets::AssetIndex;
use crate::mojang::manifest::fetch_manifest;
use crate::mojang::plan::{asset_items, core_items};
use crate::mojang::version::VersionJson;
use crate::paths::DataPaths;
use reqwest::Client;
use serde::Serialize;
use std::sync::Arc;
use tokio::fs;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallStage {
    Resolving,
    Downloading,
    Done,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub stage: InstallStage,
    pub done_files: u64,
    pub total_files: u64,
    pub done_bytes: u64,
    pub total_bytes: u64,
}

impl InstallProgress {
    fn stage(stage: InstallStage) -> Self {
        Self {
            stage,
            done_files: 0,
            total_files: 0,
            done_bytes: 0,
            total_bytes: 0,
        }
    }
}

pub async fn install_version(
    client: &Client,
    paths: &DataPaths,
    version_id: &str,
    cancel: &CancellationToken,
    on_progress: impl Fn(InstallProgress),
) -> AppResult<()> {
    on_progress(InstallProgress::stage(InstallStage::Resolving));

    let manifest = fetch_manifest(client).await?;
    let entry = manifest.find(version_id)?;
    let raw = client.get(&entry.url).send().await?.error_for_status()?.bytes().await?;
    if !sha1_bytes(&raw).eq_ignore_ascii_case(&entry.sha1) {
        return Err(AppError::Checksum(entry.url.clone()));
    }
    let version: VersionJson = serde_json::from_slice(&raw)?;

    let index_path = paths.asset_index(&version.asset_index.id);
    let index_item = DownloadItem {
        url: version.asset_index.url.clone(),
        path: index_path.clone(),
        checksum: Some(Checksum::Sha1(version.asset_index.sha1.clone())),
        size: Some(version.asset_index.size),
    };
    download_all(client, vec![index_item], cancel, &Arc::default()).await?;
    let index: AssetIndex = serde_json::from_slice(&fs::read(&index_path).await?)?;

    let mut items = core_items(&version, paths);
    items.extend(asset_items(&index, paths));
    let total_files = items.len() as u64;
    let total_bytes = items.iter().filter_map(|item| item.size).sum();

    download_with_progress(client, items, cancel, |counters| {
        on_progress(InstallProgress {
            stage: InstallStage::Downloading,
            done_files: counters.files(),
            total_files,
            done_bytes: counters.bytes(),
            total_bytes,
        });
    })
    .await?;

    write_version_json(paths, version_id, &raw).await?;
    on_progress(InstallProgress {
        stage: InstallStage::Done,
        done_files: total_files,
        total_files,
        done_bytes: total_bytes,
        total_bytes,
    });
    Ok(())
}

async fn write_version_json(paths: &DataPaths, version_id: &str, raw: &[u8]) -> AppResult<()> {
    fs::create_dir_all(paths.version_dir(version_id)).await?;
    let target = paths.version_json(version_id);
    let temp = target.with_extension("json.tmp");
    fs::write(&temp, raw).await?;
    fs::rename(&temp, &target).await?;
    Ok(())
}

pub async fn list_installed(paths: &DataPaths) -> AppResult<Vec<String>> {
    let Ok(mut entries) = fs::read_dir(paths.versions()).await else {
        return Ok(Vec::new());
    };
    let mut installed = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let id = entry.file_name().to_string_lossy().into_owned();
        if fs::try_exists(paths.version_json(&id)).await? {
            installed.push(id);
        }
    }
    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::build_client;
    use std::sync::Mutex;

    #[tokio::test]
    #[ignore = "downloads about 850 MB from Mojang servers"]
    async fn installs_a_modern_version_with_separate_native_libraries() {
        let root = std::env::temp_dir().join(format!("zenith-e2e-modern-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let paths = DataPaths::new(root.clone());
        let client = build_client().unwrap();

        install_version(&client, &paths, "1.21.1", &CancellationToken::new(), |_| {})
            .await
            .unwrap();

        let natives = walk(&paths.libraries())
            .into_iter()
            .filter(|path| path.to_string_lossy().contains("natives-windows"))
            .count();
        assert!(natives >= 1, "expected Windows LWJGL natives to be downloaded");
        assert!(paths.client_jar("1.21.1").exists());
        println!("native jars: {natives}");
        let _ = std::fs::remove_dir_all(root);
    }

    fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .flat_map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path)
                } else {
                    vec![path]
                }
            })
            .collect()
    }

    #[tokio::test]
    #[ignore = "downloads from Mojang servers"]
    async fn installs_a_real_version_and_reuses_files_on_the_second_run() {
        let root = std::env::temp_dir().join(format!("zenith-e2e-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let paths = DataPaths::new(root.clone());
        let client = build_client().unwrap();
        let stages: Mutex<Vec<InstallProgress>> = Mutex::default();

        install_version(&client, &paths, "1.0", &CancellationToken::new(), |progress| {
            stages.lock().unwrap().push(progress)
        })
        .await
        .unwrap();

        let last = stages.lock().unwrap().last().cloned().unwrap();
        assert!(matches!(last.stage, InstallStage::Done));
        assert_eq!(last.done_files, last.total_files);
        assert_eq!(last.done_bytes, last.total_bytes);
        assert!(paths.client_jar("1.0").exists());
        assert_eq!(list_installed(&paths).await.unwrap(), vec!["1.0".to_owned()]);

        let started = std::time::Instant::now();
        install_version(&client, &paths, "1.0", &CancellationToken::new(), |_| {})
            .await
            .unwrap();
        println!("second run took {:?}", started.elapsed());

        let _ = std::fs::remove_dir_all(root);
    }
}
