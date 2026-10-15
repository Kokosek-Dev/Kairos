use super::blocking;
use crate::bedrock::{self, BedrockInstance};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn list_bedrock_instances(state: State<'_, AppState>) -> AppResult<Vec<BedrockInstance>> {
    let paths = state.paths.clone();
    blocking(move || bedrock::list(&paths)).await
}

#[tauri::command]
pub async fn create_bedrock_instance(state: State<'_, AppState>, name: String) -> AppResult<BedrockInstance> {
    let paths = state.paths.clone();
    blocking(move || bedrock::create(&paths, &name)).await
}

#[tauri::command]
pub async fn delete_bedrock_instance(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || bedrock::delete(&paths, &id)).await
}

/// Asks for a pack file and adds what is inside to the instance. Nothing happens when the dialog is closed.
#[tauri::command]
pub async fn add_bedrock_pack(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<Option<BedrockInstance>> {
    let paths = state.paths.clone();
    blocking(move || {
        let chosen = app.dialog().file().add_filter("Bedrock pack", &["mcpack", "mcaddon", "zip"]).blocking_pick_file();
        let Some(chosen) = chosen else { return Ok(None) };
        let file = chosen.into_path().map_err(|error| AppError::Internal(error.to_string()))?;
        bedrock::add_pack_file(&paths, &id, &file).map(Some)
    })
    .await
}

#[tauri::command]
pub async fn remove_bedrock_pack(state: State<'_, AppState>, id: String, folder: String) -> AppResult<BedrockInstance> {
    let paths = state.paths.clone();
    blocking(move || bedrock::remove_pack(&paths, &id, &folder)).await
}

/// Puts the packs of the instance where the game lists them and starts the game.
#[tauri::command]
pub async fn play_bedrock_instance(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || {
        let game_dir = bedrock::game_data_dir().ok_or_else(|| AppError::InvalidInput("Bedrock Edition has not been started on this computer yet".into()))?;
        bedrock::apply(&paths, &id, &game_dir)?;
        app.opener().open_url("minecraft:", None::<&str>).map_err(|error| AppError::Internal(error.to_string()))
    })
    .await
}
