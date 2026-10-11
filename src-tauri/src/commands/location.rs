use super::blocking;
use crate::error::{AppError, AppResult};
use crate::location;
use crate::state::AppState;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

const FOLDER_NAME: &str = "Zenith";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataLocation {
    path: String,
    default_path: String,
    is_default: bool,
    size_bytes: u64,
}

#[tauri::command]
pub async fn get_data_location(app: AppHandle, state: State<'_, AppState>) -> AppResult<DataLocation> {
    let default = app.path().app_data_dir().map_err(|error| AppError::Internal(error.to_string()))?;
    let current = state.paths.root().to_owned();
    blocking(move || {
        Ok(DataLocation {
            path: current.to_string_lossy().into_owned(),
            default_path: default.to_string_lossy().into_owned(),
            is_default: current == default,
            size_bytes: location::size_of(&current),
        })
    })
    .await
}

/// Asks for a folder and returns where the data would live inside it.
#[tauri::command]
pub async fn pick_data_folder(app: AppHandle) -> AppResult<Option<String>> {
    blocking(move || {
        let chosen = app.dialog().file().blocking_pick_folder();
        chosen
            .map(|folder| folder.into_path().map(|path| path.join(FOLDER_NAME).to_string_lossy().into_owned()))
            .transpose()
            .map_err(|error| AppError::Internal(error.to_string()))
    })
    .await
}

#[tauri::command]
pub async fn move_data_location(app: AppHandle, state: State<'_, AppState>, folder: String) -> AppResult<()> {
    if state.is_busy() {
        return Err(AppError::InvalidInput("finish or stop the running game and downloads first".into()));
    }
    let config_dir = app.path().app_config_dir().map_err(|error| AppError::Internal(error.to_string()))?;
    let current = state.paths.root().to_owned();
    blocking(move || location::move_data(&config_dir, &current, &PathBuf::from(folder)).map(|_| ())).await
}

/// The new data folder takes effect on the next start.
#[tauri::command]
pub fn restart_app(app: AppHandle) {
    app.restart();
}
