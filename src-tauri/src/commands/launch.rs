use crate::error::{AppError, AppResult};
use crate::instances::store;
use crate::launch::{self, LaunchEvent, LaunchRequest};
use crate::app_id;
use crate::modrinth::content::{self, ContentKind};
use crate::presence;
use crate::state::AppState;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{ipc::Channel, State};

#[tauri::command]
pub async fn launch_instance(
    state: State<'_, AppState>,
    instance_id: String,
    offline: bool,
    show_on_discord: bool,
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
    let prepared = state.paths.clone();
    let prepared_id = instance.id.clone();
    crate::tasks::blocking(move || Ok(crate::texturepacks::prepare_for_launch(&prepared, &prepared_id))).await?;
    let (run_id, token) = state.runs.register(&instance.id);
    let started = Arc::new(Mutex::new(None::<Instant>));
    let paths = state.paths.clone();
    let played_id = instance.id.clone();
    let presence = state.presence.clone();
    let activity = show_on_discord.then(|| {
        let mods = content::load_entries(&state.paths, &instance.id)
            .map_or(0, |entries| entries.iter().filter(|entry| entry.kind == ContentKind::Mod).count());
        presence::describe(&instance, mods)
    });
    let emit = move |event: LaunchEvent| {
        match &event {
            LaunchEvent::Running { .. } => {
                *started.lock().unwrap() = Some(Instant::now());
                if let Some((details, line)) = &activity {
                    presence.show(details, line, chrono::Utc::now().timestamp());
                }
            }
            LaunchEvent::Exited { .. } => {
                presence.clear();
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
