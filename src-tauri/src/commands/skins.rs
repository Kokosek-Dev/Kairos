use super::blocking;
use crate::app_id;
use crate::error::{AppError, AppResult};
use crate::skins::api::ProfileView;
use crate::skins::library::{self, SavedSkin, SkinVariant};
use crate::state::AppState;
use base64::Engine;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSkinView {
    #[serde(flatten)]
    skin: SavedSkin,
    data_url: String,
}

fn to_view(skin: SavedSkin, png: &[u8]) -> SavedSkinView {
    let data_url = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png));
    SavedSkinView { skin, data_url }
}

async fn token(state: &State<'_, AppState>) -> AppResult<String> {
    Ok(state.auth.game_session(app_id::client_id).await?.access_token)
}

#[tauri::command]
pub async fn get_profile(state: State<'_, AppState>) -> AppResult<ProfileView> {
    state.skins.profile(&token(&state).await?).await
}

#[tauri::command]
pub async fn list_skins(state: State<'_, AppState>) -> AppResult<Vec<SavedSkinView>> {
    let paths = state.paths.clone();
    blocking(move || {
        library::list(&paths)?
            .into_iter()
            .map(|skin| {
                let (skin, png) = library::get(&paths, &skin.id)?;
                Ok(to_view(skin, &png))
            })
            .collect()
    })
    .await
}

#[tauri::command]
pub async fn add_skin_from_file(
    state: State<'_, AppState>,
    path: String,
    name: Option<String>,
    variant: SkinVariant,
) -> AppResult<SavedSkinView> {
    let paths = state.paths.clone();
    blocking(move || {
        let path = PathBuf::from(path);
        let name = name.unwrap_or_else(|| file_stem(&path));
        let png = std::fs::read(&path)?;
        let skin = library::add(&paths, &name, variant, &png)?;
        Ok(to_view(skin, &png))
    })
    .await
}

#[tauri::command]
pub async fn save_skin_png(
    state: State<'_, AppState>,
    name: String,
    variant: SkinVariant,
    png_base64: String,
) -> AppResult<SavedSkinView> {
    let png = base64::engine::general_purpose::STANDARD
        .decode(png_base64)
        .map_err(|_| AppError::InvalidInput("the skin image is not valid".to_owned()))?;
    let paths = state.paths.clone();
    blocking(move || {
        let skin = library::add(&paths, &name, variant, &png)?;
        Ok(to_view(skin, &png))
    })
    .await
}

fn file_stem(path: &Path) -> String {
    path.file_stem().map_or_else(|| "Skin".to_owned(), |stem| stem.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn add_skin_from_player(state: State<'_, AppState>, name: String) -> AppResult<SavedSkinView> {
    let player = state.skins.player_skin(name.trim()).await?;
    let paths = state.paths.clone();
    blocking(move || {
        let skin = library::add(&paths, &player.name, player.variant, &player.png)?;
        Ok(to_view(skin, &player.png))
    })
    .await
}

#[tauri::command]
pub async fn save_current_skin(state: State<'_, AppState>, name: String) -> AppResult<SavedSkinView> {
    let profile = state.skins.profile(&token(&state).await?).await?;
    let url = profile.skin_url.ok_or(AppError::SkinNotFound)?;
    let png = state.skins.download_texture(&url).await?;
    let paths = state.paths.clone();
    blocking(move || {
        let skin = library::add(&paths, &name, profile.variant, &png)?;
        Ok(to_view(skin, &png))
    })
    .await
}

#[tauri::command]
pub async fn remove_skin(state: State<'_, AppState>, skin_id: String) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || library::remove(&paths, &skin_id)).await
}

#[tauri::command]
pub async fn apply_skin(state: State<'_, AppState>, skin_id: String) -> AppResult<ProfileView> {
    let paths = state.paths.clone();
    let (skin, png) = blocking(move || library::get(&paths, &skin_id)).await?;
    state.skins.upload_skin(&token(&state).await?, png, skin.variant).await
}

#[tauri::command]
pub async fn reset_skin(state: State<'_, AppState>) -> AppResult<ProfileView> {
    state.skins.reset_skin(&token(&state).await?).await
}

#[tauri::command]
pub async fn set_cape(state: State<'_, AppState>, cape_id: Option<String>) -> AppResult<ProfileView> {
    state.skins.set_cape(&token(&state).await?, cape_id.as_deref()).await
}

#[tauri::command]
pub async fn pick_skin_file(app: AppHandle) -> AppResult<Option<String>> {
    blocking(move || {
        let file = app.dialog().file().add_filter("Skin", &["png"]).blocking_pick_file();
        file.map(|file| file.into_path().map(|path| path.to_string_lossy().into_owned()))
            .transpose()
            .map_err(|error| AppError::Internal(error.to_string()))
    })
    .await
}
