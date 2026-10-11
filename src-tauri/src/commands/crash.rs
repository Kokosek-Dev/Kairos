use super::blocking;
use crate::crash::{self, CrashAnalysis};
use crate::{diagnostics, memory};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn analyze_crash(state: State<'_, AppState>, instance_id: String, started_ms: u64) -> AppResult<Option<CrashAnalysis>> {
    let paths = state.paths.clone();
    let started = SystemTime::UNIX_EPOCH + Duration::from_millis(started_ms);
    blocking(move || crash::analyze(&paths, &instance_id, started)).await
}

/// Shows a file of the launcher in the file manager. Only files inside the data folder are allowed.
#[tauri::command]
pub async fn reveal_file(app: AppHandle, state: State<'_, AppState>, path: String) -> AppResult<()> {
    let root = state.paths.root().canonicalize()?;
    let file = PathBuf::from(path).canonicalize()?;
    if !file.starts_with(&root) {
        return Err(AppError::InvalidInput("that file is outside the launcher folder".into()));
    }
    app.opener().reveal_item_in_dir(file).map_err(|error| AppError::Internal(error.to_string()))
}

/// Saves a zip with logs and an overview of the setup, with sign-in values and the user name removed.
#[tauri::command]
pub async fn save_problem_report(app: AppHandle, state: State<'_, AppState>, instance_id: Option<String>) -> AppResult<Option<String>> {
    let paths = state.paths.clone();
    let version = app.package_info().version.to_string();
    blocking(move || {
        let chosen = app.dialog().file().add_filter("Zip", &["zip"]).set_file_name("kairos-problem-report.zip").blocking_save_file();
        let Some(chosen) = chosen else { return Ok(None) };
        let destination = chosen.into_path().map_err(|error| AppError::Internal(error.to_string()))?;
        diagnostics::write_report(&paths, instance_id.as_deref(), &destination, &version, memory::total_mb())?;
        Ok(Some(destination.to_string_lossy().into_owned()))
    })
    .await
}
