use crate::auth::store::{Account, Accounts};
use crate::auth::SignInEvent;
use crate::error::{AppError, AppResult};
use crate::settings;
use crate::state::AppState;
use crate::tasks::blocking;
use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, State};
use tauri_plugin_opener::OpenerExt;

const TRUSTED_LOGIN_HOSTS: [&str; 5] = ["microsoft.com", "microsoftonline.com", "live.com", "azure.com", "aka.ms"];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    client_id: Option<String>,
    configured: bool,
}

#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> AppResult<Accounts> {
    state.auth.accounts().await
}

#[tauri::command]
pub async fn sign_in(state: State<'_, AppState>, on_event: Channel<SignInEvent>) -> AppResult<Account> {
    let client_id = settings::client_id(&state.paths)?;
    let token = state.begin_sign_in()?;
    let result = state
        .auth
        .sign_in(&client_id, &token, |event| {
            let _ = on_event.send(event);
        })
        .await;
    state.end_sign_in();
    result
}

#[tauri::command]
pub fn cancel_sign_in(state: State<'_, AppState>) {
    state.cancel_sign_in();
}

#[tauri::command]
pub async fn set_active_account(state: State<'_, AppState>, account_id: String) -> AppResult<()> {
    state.auth.set_active(account_id).await
}

#[tauri::command]
pub async fn remove_account(state: State<'_, AppState>, account_id: String) -> AppResult<()> {
    state.auth.remove(account_id).await
}

#[tauri::command]
pub async fn get_auth_config(state: State<'_, AppState>) -> AppResult<AuthConfig> {
    let paths = state.paths.clone();
    blocking(move || {
        Ok(AuthConfig {
            client_id: settings::saved_client_id(&paths)?,
            configured: settings::client_id(&paths).is_ok(),
        })
    })
    .await
}

#[tauri::command]
pub async fn set_ms_client_id(state: State<'_, AppState>, client_id: Option<String>) -> AppResult<()> {
    let paths = state.paths.clone();
    blocking(move || settings::set_client_id(&paths, client_id)).await
}

#[tauri::command]
pub fn open_login_page(app: AppHandle, url: String) -> AppResult<()> {
    let parsed = reqwest::Url::parse(&url).map_err(|_| AppError::InvalidInput("invalid link".into()))?;
    let trusted = parsed.scheme() == "https"
        && parsed.host_str().is_some_and(|host| {
            TRUSTED_LOGIN_HOSTS
                .iter()
                .any(|trusted| host == *trusted || host.ends_with(&format!(".{trusted}")))
        });
    if !trusted {
        return Err(AppError::InvalidInput("this link cannot be opened from the launcher".into()));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| AppError::Internal(error.to_string()))
}
