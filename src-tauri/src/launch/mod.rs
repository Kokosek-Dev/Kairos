mod command;
mod legacy_assets;
mod log;
mod natives;
mod process;
pub mod session;

use crate::error::{AppError, AppResult};
use crate::install::{self, InstallProgress};
use crate::instances::model::Instance;
use crate::java;
use crate::mojang::assets::AssetIndex;
use crate::mojang::version::VersionJson;
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

    let version: VersionJson = serde_json::from_slice(&fs::read(paths.version_json(version_id)).await?)?;

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
            java::ensure(client, paths, version.java_major(), &token, |done, total| {
                emit(preparing(PrepareStep::Java, done, total));
            })
            .await?
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
        store::create(paths, NewInstance { name: "test".into(), game_version: version.into(), icon }).unwrap()
    }

    #[tokio::test]
    #[ignore = "downloads the game and a Java runtime, then starts the game"]
    async fn launches_the_game_and_streams_its_log() {
        let root = std::env::temp_dir().join(format!("zenith-e2e-launch-{}", std::process::id()));
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
                matches!(event, LaunchEvent::Log { line, .. } if line.contains("Setting user: ZenithDev"))
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
        let root = std::env::temp_dir().join(format!("zenith-e2e-legacy-{}", std::process::id()));
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
    }}
