use crate::error::{AppError, AppResult};
use crate::instances::store;
use crate::launch::{self, LaunchEvent, LaunchRequest};
use crate::app_id;
use crate::state::AppState;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{ipc::Channel, State};

#[tauri::command]
pub async fn launch_instance(
    state: State<'_, AppState>,
    instance_id: String,
    offline: bool,
    on_event: Channel<LaunchEvent>,
) -> AppResult<()> {
    let instance = store::get(&state.paths, &instance_id)?;
    if state.runs.is_instance_running(&instance.id) {
        return Err(AppError::InstanceRunning);
    }
    let session = match launch::session::development_override() {
        Some(session) => session,
        None if offline => state.auth.offline_session().await?,
        None => state.auth.game_session(app_id::client_id).await?,
    };
    let (run_id, token) = state.runs.register(&instance.id);
    let started = Arc::new(Mutex::new(None::<Instant>));
    let paths = state.paths.clone();
    let played_id = instance.id.clone();
    let emit = move |event: LaunchEvent| {
        match &event {
            LaunchEvent::Running { .. } => *started.lock().unwrap() = Some(Instant::now()),
            LaunchEvent::Exited { .. } => {
                if let Some(start) = started.lock().unwrap().take() {
                    let _ = store::add_play_time(&paths, &played_id, start.elapsed().as_secs());
                }
            }
            _ => {}
        }
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
