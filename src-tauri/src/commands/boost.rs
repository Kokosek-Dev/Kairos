use super::blocking;
use crate::boost::{performance_mods, with_java_flags, BoostItem, BoostReport};
use crate::error::{AppError, AppResult};
use crate::install::{InstallProgress, InstallStage};
use crate::instances::model::{Instance, InstanceSettings, LoaderKind};
use crate::instances::store;
use crate::memory;
use crate::modrinth::install::{self, Context};
use crate::moments::{self, Reason};
use crate::paths::DataPaths;
use crate::state::AppState;
use tauri::{ipc::Channel, State};
use tokio_util::sync::CancellationToken;

/// Saves a moment, then raises the memory to the recommended amount and adds the Java settings.
fn tune_settings(paths: &DataPaths, instance_id: &str) -> AppResult<(Instance, bool)> {
    moments::create(paths, instance_id, Reason::BeforeChange, "", false)?;
    let instance = store::get(paths, instance_id)?;
    let flags = with_java_flags(&instance.jvm_args);
    let flags_added = flags != instance.jvm_args;
    let recommended = memory::advise(memory::total_mb(), instance.loader != LoaderKind::Vanilla).recommended_mb;
    let memory_mb = u32::try_from(recommended).unwrap_or(instance.memory_mb).max(instance.memory_mb);
    let tuned = store::update(
        paths,
        instance_id,
        InstanceSettings {
            name: instance.name.clone(),
            game_version: instance.game_version.clone(),
            loader: instance.loader,
            loader_version: instance.loader_version.clone(),
            memory_mb,
            jvm_args: flags,
            java_path: instance.java_path.clone(),
        },
    )?;
    Ok((tuned, flags_added))
}

async fn run(state: &State<'_, AppState>, instance_id: &str, token: &CancellationToken, on_progress: &Channel<InstallProgress>) -> AppResult<BoostReport> {
    let (paths, id) = (state.paths.clone(), instance_id.to_owned());
    let (instance, java_flags_added) = blocking(move || tune_settings(&paths, &id)).await?;

    let ctx = Context { api: &state.modrinth, paths: &state.paths };
    let wanted = performance_mods(instance.loader);
    let mut items = Vec::new();
    for (index, project) in wanted.iter().enumerate() {
        let _ = on_progress.send(InstallProgress {
            stage: InstallStage::Downloading,
            done_files: index as u64,
            total_files: wanted.len() as u64,
            done_bytes: 0,
            total_bytes: 0,
        });
        match install::install_project(&ctx, &instance, project, None, token, |_, _| {}).await {
            Ok(report) => items.push(BoostItem {
                project: (*project).to_owned(),
                title: report.installed.iter().find(|entry| !entry.dependency).map(|entry| entry.title.clone()),
                installed: true,
                reason: None,
            }),
            Err(AppError::Cancelled) => return Err(AppError::Cancelled),
            Err(error) => items.push(BoostItem { project: (*project).to_owned(), title: None, installed: false, reason: Some(error.to_string()) }),
        }
    }
    Ok(BoostReport { items, memory_mb: instance.memory_mb, java_flags_added })
}

/// One click for a smoother game: performance mods for the loader, a sensible amount of memory and Java settings.
#[tauri::command]
pub async fn boost_instance(state: State<'_, AppState>, instance_id: String, on_progress: Channel<InstallProgress>) -> AppResult<BoostReport> {
    if state.runs.is_instance_running(&instance_id) {
        return Err(AppError::InstanceRunning);
    }
    let job = format!("boost:{instance_id}");
    let token = state.register_job(&job)?;
    let _ = on_progress.send(InstallProgress { stage: InstallStage::Resolving, done_files: 0, total_files: 0, done_bytes: 0, total_bytes: 0 });
    let result = run(&state, &instance_id, &token, &on_progress).await;
    state.finish_job(&job);
    let _ = on_progress.send(InstallProgress { stage: InstallStage::Done, done_files: 0, total_files: 0, done_bytes: 0, total_bytes: 0 });
    result
}

/// The performance mods that Boost installs for a loader, so the screen can name them before anything happens.
#[tauri::command]
pub fn boost_mods(loader: LoaderKind) -> Vec<&'static str> {
    performance_mods(loader).to_vec()
}

#[tauri::command]
pub fn discord_available(state: State<'_, AppState>) -> bool {
    state.presence.is_available()
}
