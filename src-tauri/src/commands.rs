use crate::error::AppResult;
use crate::install::{self, InstallProgress};
use crate::launch::{self, LaunchEvent, LaunchRequest};
use crate::mojang::manifest::{fetch_manifest, VersionManifest};
use crate::state::AppState;
use tauri::{ipc::Channel, State};
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn get_version_manifest(state: State<'_, AppState>) -> AppResult<VersionManifest> {
    fetch_manifest(&state.client).await
}

#[tauri::command]
pub async fn list_installed_versions(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    install::list_installed(&state.paths).await
}

#[tauri::command]
pub async fn install_version(
    state: State<'_, AppState>,
    version_id: String,
    on_progress: Channel<InstallProgress>,
) -> AppResult<()> {
    let token = CancellationToken::new();
    state.register_install(&version_id, token.clone())?;
    let result = install::install_version(&state.client, &state.paths, &version_id, &token, |progress| {
        let _ = on_progress.send(progress);
    })
    .await;
    state.finish_install(&version_id);
    result
}

#[tauri::command]
pub fn cancel_install(state: State<'_, AppState>, version_id: String) -> AppResult<()> {
    state.cancel_install(&version_id);
    Ok(())
}

#[tauri::command]
pub async fn launch_game(
    state: State<'_, AppState>,
    version_id: String,
    on_event: Channel<LaunchEvent>,
) -> AppResult<()> {
    let session = launch::session::current()?;
    let (run_id, token) = state.runs.register();
    let emit = move |event: LaunchEvent| {
        let _ = on_event.send(event);
    };
    emit(LaunchEvent::Queued { run_id });

    let result = launch::start(
        LaunchRequest {
            client: &state.client,
            paths: &state.paths,
            runs: &state.runs,
            session,
            version_id: &version_id,
            run_id,
            token,
        },
        emit,
    )
    .await;
    if result.is_err() {
        state.runs.finish(run_id);
    }
    result
}

#[tauri::command]
pub fn stop_game(state: State<'_, AppState>, run_id: u64) {
    state.runs.stop(run_id);
}
