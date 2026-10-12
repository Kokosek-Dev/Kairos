use super::blocking;
use crate::error::AppResult;
use crate::servers::{self, ServerEntry, ServerStatus};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_servers(state: State<'_, AppState>) -> AppResult<Vec<ServerEntry>> {
    let paths = state.paths.clone();
    blocking(move || servers::load(&paths)).await
}

#[tauri::command]
pub async fn add_server(state: State<'_, AppState>, name: String, address: String) -> AppResult<ServerEntry> {
    let paths = state.paths.clone();
    blocking(move || servers::add(&paths, &name, &address)).await
}

#[tauri::command]
pub async fn remove_server(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || servers::remove(&paths, &id)).await
}

#[tauri::command]
pub async fn ping_server(address: String) -> AppResult<ServerStatus> {
    servers::ping(&address).await
}
