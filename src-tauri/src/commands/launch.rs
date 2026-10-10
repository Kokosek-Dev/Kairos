use crate::error::{AppError, AppResult};
use crate::instances::store;
use crate::launch::{self, LaunchEvent, LaunchRequest};
use crate::state::AppState;
use tauri::{ipc::Channel, State};

#[tauri::command]
pub async fn launch_instance(
    state: State<'_, AppState>,
    instance_id: String,
    on_event: Channel<LaunchEvent>,
) -> AppResult<()> {
    let instance = store::get(&state.paths, &instance_id)?;
    if state.runs.is_instance_running(&instance.id) {
        return Err(AppError::InstanceRunning);
    }
    let session = launch::session::current()?;
    let (run_id, token) = state.runs.register(&instance.id);
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
            instance: instance.clone(),
            run_id,
            token,
        },
        emit,
    )
    .await;
    match &result {
        Ok(()) => {
            let _ = store::touch_last_played(&state.paths, &instance.id);
        }
        Err(_) => state.runs.finish(run_id),
    }
    result
}

#[tauri::command]
pub fn stop_game(state: State<'_, AppState>, run_id: u64) {
    state.runs.stop(run_id);
}
