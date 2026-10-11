use super::blocking;
use crate::error::{AppError, AppResult};
use crate::instances::model::{IconInput, Instance, InstanceSettings, LoaderKind, NewInstance};
use crate::instances::store;
use crate::memory::{self, MemoryAdvice};
use crate::paths::DataPaths;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn list_instances(state: State<'_, AppState>) -> AppResult<Vec<Instance>> {
    let paths = state.paths.clone();
    blocking(move || store::list(&paths)).await
}

#[tauri::command]
pub async fn get_instance(state: State<'_, AppState>, instance_id: String) -> AppResult<Instance> {
    let paths = state.paths.clone();
    blocking(move || store::get(&paths, &instance_id)).await
}

/// Creates an instance and gives it the amount of memory that suits this computer.
pub(super) fn create_with_advice(paths: &DataPaths, new: NewInstance) -> AppResult<Instance> {
    let modded = new.loader != LoaderKind::Vanilla;
    let created = store::create(paths, new)?;
    let advised = memory::advise(memory::total_mb(), modded).recommended_mb;
    let memory_mb = u32::try_from(advised).unwrap_or(created.memory_mb);
    store::update(
        paths,
        &created.id,
        InstanceSettings {
            name: created.name.clone(),
            game_version: created.game_version.clone(),
            loader: created.loader,
            loader_version: created.loader_version.clone(),
            memory_mb,
            jvm_args: created.jvm_args.clone(),
            java_path: created.java_path.clone(),
        },
    )
}

#[tauri::command]
pub async fn create_instance(state: State<'_, AppState>, new: NewInstance) -> AppResult<Instance> {
    let paths = state.paths.clone();
    blocking(move || create_with_advice(&paths, new)).await
}

#[tauri::command]
pub async fn update_instance(
    state: State<'_, AppState>,
    instance_id: String,
    settings: InstanceSettings,
) -> AppResult<Instance> {
    let paths = state.paths.clone();
    blocking(move || store::update(&paths, &instance_id, settings)).await
}

#[tauri::command]
pub async fn set_instance_icon(
    state: State<'_, AppState>,
    instance_id: String,
    icon: IconInput,
) -> AppResult<Instance> {
    let paths = state.paths.clone();
    blocking(move || store::set_icon(&paths, &instance_id, icon)).await
}

#[tauri::command]
pub async fn get_instance_icon(state: State<'_, AppState>, instance_id: String) -> AppResult<Option<String>> {
    let paths = state.paths.clone();
    blocking(move || store::icon_data_url(&paths, &instance_id)).await
}

#[tauri::command]
pub async fn duplicate_instance(state: State<'_, AppState>, instance_id: String) -> AppResult<Instance> {
    let paths = state.paths.clone();
    blocking(move || store::duplicate(&paths, &instance_id)).await
}

#[tauri::command]
pub async fn delete_instance(state: State<'_, AppState>, instance_id: String) -> AppResult<()> {
    if state.runs.is_instance_running(&instance_id) {
        return Err(AppError::InstanceRunning);
    }
    let paths = state.paths.clone();
    blocking(move || store::delete(&paths, &instance_id)).await
}

#[tauri::command]
pub async fn open_instance_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> AppResult<()> {
    let instance = store::get(&state.paths, &instance_id)?;
    let folder = state.paths.instance_game_dir(&instance.id);
    tokio::fs::create_dir_all(&folder).await?;
    app.opener()
        .open_path(folder.to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Internal(error.to_string()))
}

#[tauri::command]
pub async fn pick_icon_file(app: AppHandle) -> AppResult<Option<String>> {
    pick_file(app, "Images", &["png", "jpg", "jpeg", "webp", "gif"]).await
}

#[tauri::command]
pub async fn pick_java_file(app: AppHandle) -> AppResult<Option<String>> {
    let extensions: &[&str] = if cfg!(windows) { &["exe"] } else { &[] };
    pick_file(app, "Java", extensions).await
}

async fn pick_file(app: AppHandle, label: &'static str, extensions: &'static [&'static str]) -> AppResult<Option<String>> {
    blocking(move || {
        let mut dialog = app.dialog().file();
        if !extensions.is_empty() {
            dialog = dialog.add_filter(label, extensions);
        }
        let Some(file) = dialog.blocking_pick_file() else {
            return Ok(None);
        };
        let path: PathBuf = file.into_path().map_err(|error| AppError::Internal(error.to_string()))?;
        Ok(Some(path.to_string_lossy().into_owned()))
    })
    .await
}

#[tauri::command]
pub async fn get_memory_advice(modded: bool) -> AppResult<MemoryAdvice> {
    blocking(move || Ok(memory::advise(memory::total_mb(), modded))).await
}
