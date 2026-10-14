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

/// Starts Bedrock Edition and joins the server. The Microsoft Store game runs itself, so this hands it a link.
#[tauri::command]
pub fn join_bedrock(app: tauri::AppHandle, address: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let link = servers::bedrock_join_link(&address)?;
    app.opener().open_url(link, None::<&str>).map_err(|error| crate::error::AppError::Internal(error.to_string()))
}