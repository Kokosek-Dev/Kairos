use super::blocking;
use crate::error::{AppError, AppResult};
use crate::install::{InstallProgress, InstallStage};
use crate::instances::model::Instance;
use crate::instances::store;
use crate::modrinth::api::{SearchQuery, VersionFilter};
use crate::modrinth::content::{self, ContentItem, ContentKind};
use crate::modrinth::install::{self, Context, InstallReport, UpdateInfo};
use crate::modrinth::mrpack::{self, ExportOptions};
use crate::modrinth::types::{Category, Project, SearchResults, TeamMember, Version};
use crate::settings;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{ipc::Channel, AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetails {
    project: Project,
    members: Vec<TeamMember>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRequest {
    project_id: String,
    version_id: String,
}

fn progress(stage: InstallStage, done_bytes: u64, total_bytes: u64) -> InstallProgress {
    InstallProgress { stage, done_files: 0, total_files: 0, done_bytes, total_bytes }
}

fn context<'a>(state: &'a State<'_, AppState>) -> Context<'a> {
    Context { api: &state.modrinth, paths: &state.paths }
}

#[tauri::command]
pub async fn modrinth_search(state: State<'_, AppState>, query: SearchQuery) -> AppResult<SearchResults> {
    state.modrinth.search(&query).await
}

#[tauri::command]
pub async fn modrinth_project(state: State<'_, AppState>, project_id: String) -> AppResult<ProjectDetails> {
    let (project, members) = tokio::join!(state.modrinth.project(&project_id), state.modrinth.members(&project_id));
    Ok(ProjectDetails { project: project?, members: members.unwrap_or_default() })
}

#[tauri::command]
pub async fn modrinth_versions(
    state: State<'_, AppState>,
    project_id: String,
    loader: Option<String>,
    game_version: Option<String>,
) -> AppResult<Vec<Version>> {
    let filter = VersionFilter {
        loaders: loader.into_iter().collect(),
        game_versions: game_version.into_iter().collect(),
    };
    state.modrinth.versions(&project_id, &filter).await
}

#[tauri::command]
pub async fn modrinth_categories(state: State<'_, AppState>) -> AppResult<Vec<Category>> {
    state.modrinth.categories().await
}

#[tauri::command]
pub async fn list_content(state: State<'_, AppState>, instance_id: String) -> AppResult<Vec<ContentItem>> {
    let paths = state.paths.clone();
    blocking(move || content::list(&paths, &instance_id)).await
}

#[tauri::command]
pub async fn set_content_enabled(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ContentKind,
    file_name: String,
    enabled: bool,
) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || content::set_enabled(&paths, &instance_id, kind, &file_name, enabled)).await
}

#[tauri::command]
pub async fn remove_content(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ContentKind,
    file_name: String,
) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || content::remove(&paths, &instance_id, kind, &file_name)).await
}

async fn load_instance(state: &State<'_, AppState>, instance_id: &str) -> AppResult<Instance> {
    let (paths, id) = (state.paths.clone(), instance_id.to_owned());
    blocking(move || store::get(&paths, &id)).await
}

#[tauri::command]
pub async fn install_content(
    state: State<'_, AppState>,
    instance_id: String,
    project_id: String,
    version_id: Option<String>,
    on_progress: Channel<InstallProgress>,
) -> AppResult<InstallReport> {
    let instance = load_instance(&state, &instance_id).await?;
    let job = format!("modrinth:{project_id}");
    let token = state.register_job(&job)?;
    let _ = on_progress.send(progress(InstallStage::Resolving, 0, 0));
    let result = install::install_project(&context(&state), &instance, &project_id, version_id.as_deref(), &token, |done, total| {
        let _ = on_progress.send(progress(InstallStage::Downloading, done, total));
    })
    .await;
    state.finish_job(&job);
    let _ = on_progress.send(progress(InstallStage::Done, 0, 0));
    result
}

#[tauri::command]
pub async fn check_content_updates(state: State<'_, AppState>, instance_id: String) -> AppResult<Vec<UpdateInfo>> {
    let instance = load_instance(&state, &instance_id).await?;
    install::check_updates(&context(&state), &instance).await
}

#[tauri::command]
pub async fn update_content(
    state: State<'_, AppState>,
    instance_id: String,
    updates: Vec<UpdateRequest>,
    on_progress: Channel<InstallProgress>,
) -> AppResult<InstallReport> {
    let instance = load_instance(&state, &instance_id).await?;
    let job = format!("modrinth-update:{instance_id}");
    let token = state.register_job(&job)?;
    let _ = on_progress.send(progress(InstallStage::Resolving, 0, 0));

    let mut report = InstallReport::default();
    let mut outcome = Ok(());
    for update in &updates {
        let step = install::install_project(&context(&state), &instance, &update.project_id, Some(&update.version_id), &token, |done, total| {
            let _ = on_progress.send(progress(InstallStage::Downloading, done, total));
        })
        .await;
        match step {
            Ok(partial) => {
                report.installed.extend(partial.installed);
                report.missing_dependencies.extend(partial.missing_dependencies);
            }
            Err(error) => {
                outcome = Err(error);
                break;
            }
        }
    }
    state.finish_job(&job);
    let _ = on_progress.send(progress(InstallStage::Done, 0, 0));
    outcome.map(|()| report)
}

#[tauri::command]
pub async fn install_modpack(
    state: State<'_, AppState>,
    project_id: String,
    version_id: Option<String>,
    on_progress: Channel<InstallProgress>,
) -> AppResult<Instance> {
    let job = format!("modrinth:{project_id}");
    let token = state.register_job(&job)?;
    let _ = on_progress.send(progress(InstallStage::Resolving, 0, 0));
    let result = mrpack::install_modpack(&context(&state), &project_id, version_id.as_deref(), &token, |done, total| {
        let _ = on_progress.send(progress(InstallStage::Downloading, done, total));
    })
    .await;
    state.finish_job(&job);
    let _ = on_progress.send(progress(InstallStage::Done, 0, 0));
    result
}

#[tauri::command]
pub async fn import_mrpack(
    state: State<'_, AppState>,
    path: String,
    on_progress: Channel<InstallProgress>,
) -> AppResult<Instance> {
    let job = "mrpack-import";
    let token = state.register_job(job)?;
    let _ = on_progress.send(progress(InstallStage::Resolving, 0, 0));
    let result = mrpack::import(&context(&state), &PathBuf::from(path), &token, |done, total| {
        let _ = on_progress.send(progress(InstallStage::Downloading, done, total));
    })
    .await;
    state.finish_job(job);
    let _ = on_progress.send(progress(InstallStage::Done, 0, 0));
    result
}

#[tauri::command]
pub async fn export_mrpack(
    state: State<'_, AppState>,
    instance_id: String,
    destination: String,
    name: String,
    version: String,
) -> AppResult<()> {
    let instance = load_instance(&state, &instance_id).await?;
    let paths = state.paths.clone();
    blocking(move || {
        mrpack::export(&paths, &instance, &PathBuf::from(destination), &ExportOptions { name, version })
    })
    .await
}

#[tauri::command]
pub async fn pick_mrpack_file(app: AppHandle) -> AppResult<Option<String>> {
    blocking(move || {
        let file = app.dialog().file().add_filter("Modrinth modpack", &["mrpack"]).blocking_pick_file();
        file.map(|file| file.into_path().map(|path| path.to_string_lossy().into_owned()))
            .transpose()
            .map_err(|error| AppError::Internal(error.to_string()))
    })
    .await
}

#[tauri::command]
pub async fn pick_mrpack_destination(app: AppHandle, file_name: String) -> AppResult<Option<String>> {
    blocking(move || {
        let file = app
            .dialog()
            .file()
            .add_filter("Modrinth modpack", &["mrpack"])
            .set_file_name(file_name)
            .blocking_save_file();
        file.map(|file| file.into_path().map(|path| path.to_string_lossy().into_owned()))
            .transpose()
            .map_err(|error| AppError::Internal(error.to_string()))
    })
    .await
}

#[tauri::command]
pub async fn get_modrinth_contact(state: State<'_, AppState>) -> AppResult<Option<String>> {
    let paths = state.paths.clone();
    blocking(move || settings::modrinth_contact(&paths)).await
}

#[tauri::command]
pub async fn set_modrinth_contact(state: State<'_, AppState>, contact: Option<String>) -> AppResult<()> {
    let paths = state.paths.clone();
    let saved = contact.clone();
    blocking(move || settings::set_modrinth_contact(&paths, saved)).await?;
    state.modrinth.set_contact(contact.as_deref());
    Ok(())
}
