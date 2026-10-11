use super::blocking;
use super::instances::create_with_advice;
use crate::error::{AppError, AppResult};
use crate::install::{InstallProgress, InstallStage};
use crate::instances::model::{IconInput, Instance, NewInstance};
use crate::instances::store;
use crate::modrinth::content;
use crate::modrinth::install::{self, Context};
use crate::share::{self, CodePreview};
use crate::state::AppState;
use serde::Serialize;
use tauri::{ipc::Channel, State};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOutcome {
    instance: Instance,
    /// How many pieces of content could not be installed, e.g. because no version fits any more.
    failed: u32,
}

#[tauri::command]
pub async fn export_instance_code(state: State<'_, AppState>, instance_id: String) -> AppResult<String> {
    let paths = state.paths.clone();
    blocking(move || {
        let instance = store::get(&paths, &instance_id)?;
        share::encode(&instance, &content::load_entries(&paths, &instance_id)?)
    })
    .await
}

#[tauri::command]
pub async fn preview_instance_code(code: String) -> AppResult<CodePreview> {
    blocking(move || Ok(share::preview(&share::decode(&code)?))).await
}

fn progress(done: usize, total: usize) -> InstallProgress {
    InstallProgress { stage: InstallStage::Downloading, done_files: done as u64, total_files: total as u64, done_bytes: 0, total_bytes: 0 }
}

/// Builds the instance a code describes. Dependencies are installed before the mods that need them.
#[tauri::command]
pub async fn import_instance_code(state: State<'_, AppState>, code: String, on_progress: Channel<InstallProgress>) -> AppResult<ImportOutcome> {
    let payload = share::decode(&code)?;
    let job = "code-import";
    let token = state.register_job(job)?;
    let _ = on_progress.send(InstallProgress { stage: InstallStage::Resolving, done_files: 0, total_files: 0, done_bytes: 0, total_bytes: 0 });

    let paths = state.paths.clone();
    let new = NewInstance {
        name: payload.n.clone(),
        game_version: payload.g.clone(),
        icon: IconInput::Preset { key: "cube".into() },
        loader: payload.l,
        loader_version: payload.lv.clone(),
    };
    let created = blocking(move || create_with_advice(&paths, new)).await;
    let instance = match created {
        Ok(instance) => instance,
        Err(error) => {
            state.finish_job(job);
            return Err(error);
        }
    };

    let ctx = Context { api: &state.modrinth, paths: &state.paths };
    let items = share::install_order(&payload);
    let mut failed = 0u32;
    let mut cancelled = false;
    for (index, item) in items.iter().enumerate() {
        let _ = on_progress.send(progress(index, items.len()));
        match install::install_project(&ctx, &instance, item.project(), Some(item.version()), &token, |_, _| {}).await {
            Ok(_) => {}
            Err(AppError::Cancelled) => {
                cancelled = true;
                break;
            }
            Err(_) => failed += 1,
        }
    }
    state.finish_job(job);
    let _ = on_progress.send(InstallProgress { stage: InstallStage::Done, done_files: 0, total_files: 0, done_bytes: 0, total_bytes: 0 });

    if cancelled || (!items.is_empty() && failed as usize == items.len()) {
        let (paths, id) = (state.paths.clone(), instance.id.clone());
        let _ = blocking(move || store::delete(&paths, &id)).await;
        return Err(if cancelled { AppError::Cancelled } else { AppError::NoCompatibleVersion(payload.n) });
    }
    Ok(ImportOutcome { instance, failed })
}
