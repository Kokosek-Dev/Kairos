use super::endpoints::Endpoints;
use super::xbox::XboxToken;
use crate::error::{AppError, AppResult, AuthFailure};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MinecraftToken {
    pub access_token: String,
    pub lifetime: Duration,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct Entitlements {
    #[serde(default)]
    items: Vec<serde_json::Value>,
}

pub async fn login(client: &Client, endpoints: &Endpoints, xsts: &XboxToken) -> AppResult<MinecraftToken> {
    let response = client
        .post(&endpoints.minecraft_login)
        .header("Accept", "application/json")
        .json(&json!({ "identityToken": format!("XBL3.0 x={};{}", xsts.user_hash, xsts.token) }))
        .send()
        .await?;
    match response.status() {
        status if status.is_success() => {
            let parsed: LoginResponse = response.json().await?;
            Ok(MinecraftToken {
                access_token: parsed.access_token,
                lifetime: Duration::from_secs(parsed.expires_in),
            })
        }
        StatusCode::FORBIDDEN => Err(AuthFailure::AppNotApproved.into()),
        status => Err(AppError::HttpStatus {
            status: status.as_u16(),
            url: endpoints.minecraft_login.clone(),
        }),
    }
}

/// A Minecraft profile only exists for accounts that own the game, so a successful
/// profile lookup doubles as the ownership check.
pub async fn require_profile(client: &Client, endpoints: &Endpoints, access_token: &str) -> AppResult<Profile> {
    let response = client.get(&endpoints.profile).bearer_auth(access_token).send().await?;
    match response.status() {
        status if status.is_success() => Ok(response.json().await?),
        StatusCode::NOT_FOUND => {
            if owns_game(client, endpoints, access_token).await? {
                Err(AuthFailure::ProfileMissing.into())
            } else {
                Err(AuthFailure::GameNotOwned.into())
            }
        }
        status => Err(AppError::HttpStatus {
            status: status.as_u16(),
            url: endpoints.profile.clone(),
        }),
    }
}

async fn owns_game(client: &Client, endpoints: &Endpoints, access_token: &str) -> AppResult<bool> {
    let entitlements: Entitlements = client
        .get(&endpoints.entitlements)
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(!entitlements.items.is_empty())
}
