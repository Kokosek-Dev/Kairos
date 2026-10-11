mod command;
mod legacy_assets;
mod log;
mod natives;
mod process;
pub mod session;

use crate::error::{AppError, AppResult};
use crate::install::{self, InstallProgress};
use crate::download::download_with_progress;
use crate::instances::model::{Instance, LoaderKind};
use crate::java;
use crate::mojang::assets::AssetIndex;
use crate::loaders::{self, InstallContext};
use crate::mojang::plan::core_items;
use crate::mojang::profile;
use crate::paths::DataPaths;
use crate::state::RunRegistry;
use command::{build_command, LaunchSpec};
use reqwest::Client;
use serde::Serialize;
use std::path::PathBuf;
use session::GameSession;
use tokio::fs;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrepareStep {
    Version,
    Loader,
    Java,
    Files,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum LaunchEvent {
    Queued { run_id: u64 },
    Preparing { step: PrepareStep, done_bytes: u64, total_bytes: u64 },
    Running { pid: Option<u32> },
    Log { stream: LogStream, level: Option<String>, line: String },
    Exited { code: Option<i32> },
}

pub struct LaunchRequest<'a> {
    pub client: &'a Client,
    pub paths: &'a DataPaths,
    pub runs: &'a RunRegistry,
    pub session: GameSession,
    pub instance: Instance,
    pub run_id: u64,
    pub token: CancellationToken,
}

pub async fn start(
    request: LaunchRequest<'_>,
    emit: impl Fn(LaunchEvent) + Clone + Send + Sync + 'static,
) -> AppResult<()> {
    let LaunchRequest { client, paths, runs, session, instance, run_id, token } = request;
    let version_id = instance.game_version.as_str();
    let preparing = |step, done_bytes, total_bytes| LaunchEvent::Preparing { step, done_bytes, total_bytes };

    if !install::list_installed(paths).await?.iter().any(|id| id == version_id) {
        emit(preparing(PrepareStep::Version, 0, 0));
        install::install_version(client, paths, version_id, &token, |progress: InstallProgress| {
            emit(preparing(PrepareStep::Version, progress.done_bytes, progress.total_bytes));
        })
        .await?;
    }

    let vanilla = profile::load_resolved(paths, version_id).await?;

    emit(preparing(PrepareStep::Java, 0, 0));
    let java = match &instance.java_path {
        Some(custom) => {
            let custom = PathBuf::from(custom);
            if !fs::try_exists(&custom).await? {
                return Err(AppError::InvalidInput(format!("Java was not found at {}", custom.display())));
            }
            custom
        }
        None => {
            java::ensure(client, paths, vanilla.java_major(), &token, |done, total| {
                emit(preparing(PrepareStep::Java, done, total));
            })
            .await?
        }
    };

    let version = match instance.loader {
        LoaderKind::Vanilla => vanilla,
        kind => {
            emit(preparing(PrepareStep::Loader, 0, 0));
            let loader_version = loaders::require_version(kind, instance.loader_version.as_deref())?;
            let context = InstallContext { client, paths, java: &java, cancel: &token };
            let launch_id = loaders::ensure_installed(&context, kind, version_id, loader_version, &|line| {
                emit(LaunchEvent::Log { stream: LogStream::Stdout, level: None, line });
            })
            .await?;
            let merged = profile::load_resolved(paths, &launch_id).await?;
            download_with_progress(client, core_items(&merged, paths), &token, |counters| {
                emit(preparing(PrepareStep::Loader, counters.bytes(), 0));
            })
            .await?;
            merged
        }
    };

    emit(preparing(PrepareStep::Files, 0, 0));
    let game_dir = paths.instance_game_dir(&instance.id);
    fs::create_dir_all(&game_dir).await?;
    let index: AssetIndex = serde_json::from_slice(&fs::read(paths.asset_index(&version.asset_index.id)).await?)?;
    let game_assets = {
        let (version, paths, game_dir) = (version.clone(), paths.clone(), game_dir.clone());
        tokio::task::spawn_blocking(move || -> AppResult<_> {
            natives::extract_natives(&version, &paths)?;
            legacy_assets::prepare(&index, &version.asset_index.id, &paths, &game_dir)
        })
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??
    };

    if token.is_cancelled() {
        return Err(AppError::Cancelled);
    }

    let command = build_command(&LaunchSpec {
        version: &version,
        paths,
        java,
        game_dir,
        game_assets,
        memory_mb: instance.memory_mb,
        extra_jvm_args: instance.jvm_args.clone(),
        session: &session,
    });
    process::spawn(&command, run_id, token, runs.clone(), emit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::build_client;
    use crate::instances::model::{IconInput, NewInstance};
    use crate::instances::store;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn test_instance(paths: &DataPaths, version: &str) -> Instance {
        let icon = IconInput::Preset { key: "cube".into() };
        let new = NewInstance { name: "test".into(), game_version: version.into(), icon, loader: LoaderKind::Vanilla, loader_version: None };
        store::create(paths, new).unwrap()
    }

    #[tokio::test]
    #[ignore = "downloads the game and a Java runtime, then starts the game"]
    async fn launches_the_game_and_streams_its_log() {
        let root = std::env::temp_dir().join(format!("kairos-e2e-launch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let paths = DataPaths::new(root.clone());
        let client = build_client().unwrap();
        let runs = RunRegistry::default();
        let (run_id, token) = runs.register("test");
        let events: Arc<Mutex<Vec<LaunchEvent>>> = Arc::default();
        let sink = events.clone();

        start(
            LaunchRequest {
                client: &client,
                paths: &paths,
                runs: &runs,
                session: GameSession::development(),
                instance: test_instance(&paths, "1.21.1"),
                run_id,
                token,
            },
            move |event| sink.lock().unwrap().push(event),
        )
        .await
        .unwrap();

        let mut seen_user = false;
        for _ in 0..240 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            seen_user = events.lock().unwrap().iter().any(|event| {
                matches!(event, LaunchEvent::Log { line, .. } if line.contains("Setting user: KairosDev"))
            });
            if seen_user {
                break;
            }
        }
        runs.stop(run_id);
        tokio::time::sleep(Duration::from_secs(3)).await;

        let log: Vec<String> = events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|event| match event {
                LaunchEvent::Log { line, .. } => Some(line.clone()),
                _ => None,
            })
            .collect();
        println!("--- first log lines ---\n{}", log.iter().take(15).cloned().collect::<Vec<_>>().join("\n"));
        assert!(seen_user, "the game never logged the user line");
        assert!(events.lock().unwrap().iter().any(|event| matches!(event, LaunchEvent::Exited { .. })));
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    #[ignore = "downloads Java 8 and an old game version, then starts the game"]
    async fn launches_a_legacy_version_with_extracted_natives_and_mapped_resources() {
        let root = std::env::temp_dir().join(format!("kairos-e2e-legacy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let paths = DataPaths::new(root.clone());
        let client = build_client().unwrap();
        let runs = RunRegistry::default();
        let (run_id, token) = runs.register("test");
        let events: Arc<Mutex<Vec<LaunchEvent>>> = Arc::default();
        let sink = events.clone();

        start(
            LaunchRequest {
                client: &client,
                paths: &paths,
                runs: &runs,
                session: GameSession::development(),
                instance: test_instance(&paths, "1.0"),
                run_id,
                token,
            },
            move |event| sink.lock().unwrap().push(event),
        )
        .await
        .unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;

        let exited_early = events.lock().unwrap().iter().any(|event| matches!(event, LaunchEvent::Exited { .. }));
        let lines: Vec<String> = events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|event| match event {
                LaunchEvent::Log { line, .. } => Some(line.clone()),
                _ => None,
            })
            .collect();
        println!("--- legacy log ---\n{}", lines.join("\n"));
        runs.stop(run_id);
        tokio::time::sleep(Duration::from_secs(2)).await;

        let natives: Vec<_> = std::fs::read_dir(paths.natives_dir("1.0")).unwrap().flatten().collect();
        assert!(!natives.is_empty(), "natives were not extracted");
        assert!(paths.instance_game_dir("test").join("resources").exists(), "resources were not mapped");
        assert!(!exited_early, "the legacy game exited right after starting");
        let _ = std::fs::remove_dir_all(root);
    }
    fn shared_paths() -> DataPaths {
        DataPaths::new(std::env::temp_dir().join("kairos-e2e-shared"))
    }

    async fn launch_and_collect_log(loader: LoaderKind, game_version: &str, loader_version: &str, mods: &[&str]) -> Vec<String> {
        let paths = shared_paths();
        let name = format!("e2e-{loader:?}").to_lowercase();
        let _ = std::fs::remove_dir_all(paths.instance_dir(&name));
        let icon = IconInput::Preset { key: "cube".into() };
        let new = NewInstance {
            name,
            game_version: game_version.into(),
            icon,
            loader,
            loader_version: Some(loader_version.into()),
        };
        let instance = store::create(&paths, new).unwrap();

        if !mods.is_empty() {
            let api = crate::modrinth::api::ModrinthApi::new(build_client().unwrap());
            let context = crate::modrinth::install::Context { api: &api, paths: &paths };
            for slug in mods {
                crate::modrinth::install::install_project(&context, &instance, slug, None, &CancellationToken::new(), |_, _| {})
                    .await
                    .unwrap();
            }
        }

        let client = build_client().unwrap();
        let runs = RunRegistry::default();
        let (run_id, token) = runs.register(&instance.id);
        let events: Arc<Mutex<Vec<LaunchEvent>>> = Arc::default();
        let sink = events.clone();
        start(
            LaunchRequest {
                client: &client,
                paths: &paths,
                runs: &runs,
                session: GameSession::development(),
                instance,
                run_id,
                token,
            },
            move |event| sink.lock().unwrap().push(event),
        )
        .await
        .unwrap();

        let lines = |events: &Mutex<Vec<LaunchEvent>>| -> Vec<String> {
            events
                .lock()
                .unwrap()
                .iter()
                .filter_map(|event| match event {
                    LaunchEvent::Log { line, .. } => Some(line.clone()),
                    _ => None,
                })
                .collect()
        };
        for _ in 0..360 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let exited = events.lock().unwrap().iter().any(|event| matches!(event, LaunchEvent::Exited { .. }));
            if exited || lines(&events).iter().any(|line| line.contains("Setting user: KairosDev")) {
                break;
            }
        }
        runs.stop(run_id);
        tokio::time::sleep(Duration::from_secs(3)).await;
        let log = lines(&events);
        println!("--- {loader:?} log (first 40 of {}) ---\n{}", log.len(), log.iter().take(40).cloned().collect::<Vec<_>>().join("\n"));
        log
    }

    fn assert_reached_the_game(log: &[String], loader_word: &str) {
        let joined = log.join("\n").to_lowercase();
        assert!(joined.contains(loader_word), "the log never mentions {loader_word}");
        assert!(joined.contains("setting user: kairosdev"), "the game never reached Minecraft's main");
    }

    #[tokio::test]
    #[ignore = "downloads the game and Fabric, then starts the game"]
    async fn launches_a_fabric_instance() {
        let versions = crate::loaders::list_versions(&build_client().unwrap(), LoaderKind::Fabric, "1.21.1").await.unwrap();
        let recommended = versions.iter().find(|version| version.recommended).unwrap();
        let log = launch_and_collect_log(LoaderKind::Fabric, "1.21.1", &recommended.version, &[]).await;
        assert_reached_the_game(&log, "fabric");
    }

    #[tokio::test]
    #[ignore = "downloads the game and Quilt, then starts the game"]
    async fn launches_a_quilt_instance() {
        let versions = crate::loaders::list_versions(&build_client().unwrap(), LoaderKind::Quilt, "1.21.1").await.unwrap();
        let recommended = versions.iter().find(|version| version.recommended).unwrap();
        let log = launch_and_collect_log(LoaderKind::Quilt, "1.21.1", &recommended.version, &[]).await;
        assert_reached_the_game(&log, "quilt");
    }

    #[tokio::test]
    #[ignore = "downloads the game and runs the Forge installer, then starts the game"]
    async fn launches_a_forge_instance() {
        let versions = crate::loaders::list_versions(&build_client().unwrap(), LoaderKind::Forge, "1.20.1").await.unwrap();
        let recommended = versions.iter().find(|version| version.recommended).unwrap();
        let log = launch_and_collect_log(LoaderKind::Forge, "1.20.1", &recommended.version, &[]).await;
        assert_reached_the_game(&log, "forge");
    }

    #[tokio::test]
    #[ignore = "downloads the game and runs the NeoForge installer, then starts the game"]
    async fn launches_a_neoforge_instance() {
        let versions = crate::loaders::list_versions(&build_client().unwrap(), LoaderKind::NeoForge, "1.21.1").await.unwrap();
        let recommended = versions.iter().find(|version| version.recommended).unwrap();
        let log = launch_and_collect_log(LoaderKind::NeoForge, "1.21.1", &recommended.version, &[]).await;
        assert_reached_the_game(&log, "neoforge");
    }
    #[tokio::test]
    #[ignore = "downloads Java 8, an old game and the legacy Forge installer, then starts the game"]
    async fn launches_a_legacy_forge_instance() {
        let versions = crate::loaders::list_versions(&build_client().unwrap(), LoaderKind::Forge, "1.12.2").await.unwrap();
        let recommended = versions.iter().find(|version| version.recommended).unwrap();
        let log = launch_and_collect_log(LoaderKind::Forge, "1.12.2", &recommended.version, &[]).await;
        assert_reached_the_game(&log, "forge");
    }
    #[tokio::test]
    #[ignore = "downloads the game, Fabric and mods from Modrinth, then starts the game"]
    async fn mods_installed_from_modrinth_load_in_the_game() {
        let versions = crate::loaders::list_versions(&build_client().unwrap(), LoaderKind::Fabric, "1.21.1").await.unwrap();
        let recommended = versions.iter().find(|version| version.recommended).unwrap();
        let log = launch_and_collect_log(LoaderKind::Fabric, "1.21.1", &recommended.version, &["modmenu"]).await;
        let joined = log.join("\n").to_lowercase();
        assert!(joined.contains("modmenu") || joined.contains("mod menu"), "Mod Menu never showed up in the log");
        assert!(joined.contains("fabric-api") || joined.contains("fabric api"), "the Fabric API dependency never loaded");
        assert_reached_the_game(&log, "fabric");
    }}
