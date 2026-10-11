use super::blocking;
use crate::error::{AppError, AppResult};
use crate::moments::{self, Difference, Moment, Reason, Sizes};
use crate::state::AppState;
use tauri::State;

fn ensure_stopped(state: &State<'_, AppState>, instance_id: &str) -> AppResult<()> {
    if state.runs.is_instance_running(instance_id) {
        return Err(AppError::InstanceRunning);
    }
    Ok(())
}

#[tauri::command]
pub async fn list_moments(state: State<'_, AppState>, instance_id: String) -> AppResult<Vec<Moment>> {
    let paths = state.paths.clone();
    blocking(move || moments::list(&paths, &instance_id)).await
}

#[tauri::command]
pub async fn create_moment(state: State<'_, AppState>, instance_id: String, note: String, include_worlds: bool) -> AppResult<Moment> {
    let paths = state.paths.clone();
    blocking(move || moments::create(&paths, &instance_id, Reason::Manual, &note, include_worlds)).await
}

/// Called before something risky, like updating mods, so there is always a way back.
#[tauri::command]
pub async fn create_safety_moment(state: State<'_, AppState>, instance_id: String, reason: Reason) -> AppResult<Moment> {
    if !matches!(reason, Reason::BeforeUpdate | Reason::BeforeChange) {
        return Err(AppError::InvalidInput("unsupported reason".into()));
    }
    let paths = state.paths.clone();
    blocking(move || moments::create(&paths, &instance_id, reason, "", false)).await
}

#[tauri::command]
pub async fn restore_moment(state: State<'_, AppState>, instance_id: String, moment_id: String) -> AppResult<()> {
    ensure_stopped(&state, &instance_id)?;
    let paths = state.paths.clone();
    blocking(move || moments::restore(&paths, &instance_id, &moment_id)).await
}

#[tauri::command]
pub async fn delete_moment(state: State<'_, AppState>, instance_id: String, moment_id: String) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || moments::delete(&paths, &instance_id, &moment_id)).await
}

#[tauri::command]
pub async fn compare_moment(state: State<'_, AppState>, instance_id: String, moment_id: String) -> AppResult<Difference> {
    let paths = state.paths.clone();
    blocking(move || moments::compare(&paths, &instance_id, &moment_id)).await
}

#[tauri::command]
pub async fn moment_sizes(state: State<'_, AppState>, instance_id: String) -> AppResult<Sizes> {
    let paths = state.paths.clone();
    blocking(move || moments::sizes(&paths, &instance_id)).await
}
