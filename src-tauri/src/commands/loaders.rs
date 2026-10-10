use crate::error::AppResult;
use crate::instances::model::LoaderKind;
use crate::loaders::{self, LoaderVersion};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_loader_versions(
    state: State<'_, AppState>,
    loader: LoaderKind,
    game_version: String,
) -> AppResult<Vec<LoaderVersion>> {
    loaders::list_versions(&state.client, loader, &game_version).await
}
