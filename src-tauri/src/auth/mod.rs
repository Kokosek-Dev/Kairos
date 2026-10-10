pub mod endpoints;
mod loopback;
mod microsoft;
mod minecraft;
pub mod store;
mod xbox;

#[cfg(test)]
mod tests;

use crate::error::{AppError, AppResult, AuthFailure};
use crate::launch::session::GameSession;
use crate::tasks::blocking;
use chrono::Utc;
use endpoints::Endpoints;
use minecraft::MinecraftToken;
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use store::{Account, AccountStore, Accounts};
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

const REUSE_MARGIN: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignInStep {
    Xbox,
    Xsts,
    Minecraft,
    Profile,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum SignInEvent {
    Browser {
        url: String,
    },
    Step {
        step: SignInStep,
    },
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

pub struct AuthService {
    client: Client,
    endpoints: Endpoints,
    accounts: Arc<AccountStore>,
    cache: Mutex<HashMap<String, CachedToken>>,
    refresh_lock: Mutex<()>,
}

impl AuthService {
    pub fn new(client: Client, endpoints: Endpoints, accounts: AccountStore) -> Self {
        Self {
            client,
            endpoints,
            accounts: Arc::new(accounts),
            cache: Mutex::default(),
            refresh_lock: Mutex::default(),
        }
    }

    pub async fn accounts(&self) -> AppResult<Accounts> {
        let store = self.accounts.clone();
        blocking(move || store.load()).await
    }

    pub async fn set_active(&self, id: String) -> AppResult<()> {
        let store = self.accounts.clone();
        blocking(move || store.set_active(&id)).await
    }

    pub async fn remove(&self, id: String) -> AppResult<()> {
        self.cache.lock().await.remove(&id);
        let store = self.accounts.clone();
        blocking(move || store.remove(&id)).await
    }

    pub async fn sign_in(
        &self,
        client_id: &str,
        cancel: &CancellationToken,
        open_browser: impl FnOnce(&str) -> AppResult<()>,
        emit: impl Fn(SignInEvent),
    ) -> AppResult<Account> {
        let pkce = microsoft::Pkce::generate()?;
        let state = microsoft::random_state()?;
        let loopback = loopback::bind().await?;
        let redirect_uri = loopback.redirect_uri.clone();

        let url = microsoft::authorize_url(&self.endpoints, client_id, &redirect_uri, &state, &pkce)?;
        emit(SignInEvent::Browser { url: url.clone() });
        open_browser(&url)?;
        let code = loopback.wait_for_code(&state, cancel).await?;
        let tokens =
            microsoft::exchange_code(&self.client, &self.endpoints, client_id, &code, &redirect_uri, &pkce).await?;

        let minecraft = self
            .minecraft_token(&tokens.access_token, |step| emit(SignInEvent::Step { step }))
            .await?;
        emit(SignInEvent::Step { step: SignInStep::Profile });
        let profile = minecraft::require_profile(&self.client, &self.endpoints, &minecraft.access_token).await?;

        let account = Account {
            id: profile.id,
            username: profile.name,
            added_at: Utc::now(),
        };
        let store = self.accounts.clone();
        let (saved, refresh_token) = (account.clone(), tokens.refresh_token);
        blocking(move || store.upsert(saved, &refresh_token)).await?;
        self.remember(&account.id, minecraft).await;
        Ok(account)
    }

    pub async fn game_session(&self, client_id: impl FnOnce() -> AppResult<String>) -> AppResult<GameSession> {
        let store = self.accounts.clone();
        let account = blocking(move || store.active())
            .await?
            .ok_or(AppError::NotSignedIn)?;
        let access_token = self.fresh_token(client_id, &account).await?;
        Ok(GameSession {
            username: account.username,
            uuid: account.id,
            access_token,
            user_type: "msa".into(),
            xuid: "0".into(),
            client_id: "0".into(),
        })
    }

    async fn fresh_token(&self, client_id: impl FnOnce() -> AppResult<String>, account: &Account) -> AppResult<String> {
        let _guard = self.refresh_lock.lock().await;
        if let Some(cached) = self.cache.lock().await.get(&account.id) {
            if cached.expires_at > Instant::now() + REUSE_MARGIN {
                return Ok(cached.access_token.clone());
            }
        }

        let store = self.accounts.clone();
        let id = account.id.clone();
        let refresh_token = blocking(move || store.refresh_token(&id))
            .await?
            .ok_or(AuthFailure::SessionExpired)?;
        let tokens = microsoft::refresh(&self.client, &self.endpoints, &client_id()?, &refresh_token).await?;

        let store = self.accounts.clone();
        let (id, rotated) = (account.id.clone(), tokens.refresh_token.clone());
        blocking(move || store.save_refresh_token(&id, &rotated)).await?;

        let minecraft = self.minecraft_token(&tokens.access_token, |_| {}).await?;
        let access_token = minecraft.access_token.clone();
        self.remember(&account.id, minecraft).await;
        Ok(access_token)
    }

    async fn minecraft_token(
        &self,
        microsoft_access_token: &str,
        on_step: impl Fn(SignInStep),
    ) -> AppResult<MinecraftToken> {
        on_step(SignInStep::Xbox);
        let user = xbox::authenticate(&self.client, &self.endpoints, microsoft_access_token).await?;
        on_step(SignInStep::Xsts);
        let xsts = xbox::authorize(&self.client, &self.endpoints, &user.token).await?;
        on_step(SignInStep::Minecraft);
        minecraft::login(&self.client, &self.endpoints, &xsts).await
    }

    async fn remember(&self, account_id: &str, token: MinecraftToken) {
        self.cache.lock().await.insert(
            account_id.to_owned(),
            CachedToken {
                access_token: token.access_token,
                expires_at: Instant::now() + token.lifetime,
            },
        );
    }
}
