use super::blocking;
use crate::error::AppResult;
use crate::servers::{self, Edition, ServerEntry, ServerStatus};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_servers(state: State<'_, AppState>) -> AppResult<Vec<ServerEntry>> {
    let paths = state.paths.clone();
    blocking(move || servers::load(&paths)).await
}

#[tauri::command]
pub async fn add_server(state: State<'_, AppState>, name: String, address: String, edition: Edition) -> AppResult<ServerEntry> {
    let paths = state.paths.clone();
    blocking(move || servers::add(&paths, &name, &address, edition)).await
}

#[tauri::command]
pub async fn remove_server(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || servers::remove(&paths, &id)).await
}

#[tauri::command]
pub async fn ping_server(address: String, edition: Edition) -> AppResult<ServerStatus> {
    servers::ping(&address, edition).await
}

/// Whether Minecraft Bedrock Edition from the Microsoft Store is installed for this user.
#[tauri::command]
pub fn bedrock_installed() -> bool {
    servers::bedrock_installed()
}

/// Starts Bedrock Edition. It opens its own launcher, because the Microsoft Store runs that game.
#[tauri::command]
pub fn open_bedrock(app: tauri::AppHandle) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url("minecraft:", None::<&str>).map_err(|error| crate::error::AppError::Internal(error.to_string()))
}