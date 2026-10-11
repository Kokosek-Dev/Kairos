use super::blocking;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::texturepacks::{self, PackInfo, TextureList};
use base64::Engine;
use std::path::PathBuf;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn list_texture_versions(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    let paths = state.paths.clone();
    blocking(move || texturepacks::available_versions(&paths)).await
}

#[tauri::command]
pub async fn create_texture_pack(state: State<'_, AppState>, name: String, game_version: String) -> AppResult<PackInfo> {
    let paths = state.paths.clone();
    blocking(move || texturepacks::create(&paths, &name, &game_version)).await
}

#[tauri::command]
pub async fn list_texture_packs(state: State<'_, AppState>) -> AppResult<Vec<PackInfo>> {
    let paths = state.paths.clone();
    blocking(move || texturepacks::list(&paths)).await
}

#[tauri::command]
pub async fn delete_texture_pack(state: State<'_, AppState>, pack_id: String) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || texturepacks::delete(&paths, &pack_id)).await
}

#[tauri::command]
pub async fn list_pack_textures(state: State<'_, AppState>, pack_id: String) -> AppResult<TextureList> {
    let paths = state.paths.clone();
    blocking(move || texturepacks::list_textures(&paths, &pack_id)).await
}

#[tauri::command]
pub async fn read_pack_texture(state: State<'_, AppState>, pack_id: String, path: String) -> AppResult<String> {
    let paths = state.paths.clone();
    let png = blocking(move || texturepacks::read_texture(&paths, &pack_id, &path)).await?;
    Ok(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png)))
}

#[tauri::command]
pub async fn write_pack_texture(
    state: State<'_, AppState>,
    pack_id: String,
    path: String,
    png_base64: String,
) -> AppResult<()> {
    let png = base64::engine::general_purpose::STANDARD
        .decode(png_base64)
        .map_err(|_| AppError::InvalidInput("the texture image is not valid".to_owned()))?;
    let paths = state.paths.clone();
    blocking(move || texturepacks::write_texture(&paths, &pack_id, &path, &png)).await
}

#[tauri::command]
pub async fn reset_pack_texture(state: State<'_, AppState>, pack_id: String, path: String) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || texturepacks::reset_texture(&paths, &pack_id, &path)).await
}

#[tauri::command]
pub async fn export_texture_pack(app: AppHandle, state: State<'_, AppState>, pack_id: String) -> AppResult<Option<String>> {
    let paths = state.paths.clone();
    blocking(move || {
        let pack = texturepacks::info(&paths, &pack_id)?;
        let chosen = app
            .dialog()
            .file()
            .add_filter("Resource pack", &["zip"])
            .set_file_name(format!("{}.zip", pack.name))
            .blocking_save_file();
        let Some(chosen) = chosen else { return Ok(None) };
        let destination: PathBuf = chosen.into_path().map_err(|error| AppError::Internal(error.to_string()))?;
        texturepacks::export(&paths, &pack_id, &destination)?;
        Ok(Some(destination.to_string_lossy().into_owned()))
    })
    .await
}

#[tauri::command]
pub async fn install_texture_pack(state: State<'_, AppState>, pack_id: String, instance_id: String) -> AppResult<String> {
    let paths = state.paths.clone();
    let installed = blocking(move || texturepacks::install(&paths, &pack_id, &instance_id)).await?;
    Ok(installed.to_string_lossy().into_owned())
}
