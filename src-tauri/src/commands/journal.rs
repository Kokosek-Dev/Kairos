use super::blocking;
use crate::error::AppResult;
use crate::journal::{self, Session};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> AppResult<Vec<Session>> {
    let paths = state.paths.clone();
    blocking(move || journal::load(&paths)).await
}
