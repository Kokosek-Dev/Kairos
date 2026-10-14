use super::blocking;
use crate::error::AppResult;
use crate::feedback::{self, Report};
use crate::state::AppState;
use crate::{diagnostics, memory};
use tauri::{AppHandle, State};

/// Whether this build can deliver feedback by itself; without it the panel offers copying instead.
#[tauri::command]
pub fn feedback_available() -> bool {
    feedback::is_available()
}

/// The technical details that would be sent along, so the player can read them first.
#[tauri::command]
pub async fn feedback_details(app: AppHandle, state: State<'_, AppState>, instance_id: Option<String>) -> AppResult<String> {
    let paths = state.paths.clone();
    let version = app.package_info().version.to_string();
    blocking(move || diagnostics::overview(&paths, instance_id.as_deref(), &version, memory::total_mb())).await
}

#[tauri::command]
pub async fn send_feedback(app: AppHandle, state: State<'_, AppState>, report: Report) -> AppResult<()> {
    feedback::submit(&state.client, report, &app.package_info().version.to_string()).await
}
