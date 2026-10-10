use super::endpoints::Endpoints;
use crate::error::{AppError, AppResult, AuthFailure};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct XboxToken {
    pub token: String,
    pub user_hash: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct XboxResponse {
    token: String,
    display_claims: DisplayClaims,
}

#[derive(Debug, Deserialize)]
struct DisplayClaims {
    xui: Vec<UserInfo>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    uhs: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct XboxErrorBody {
    x_err: u64,
}

fn failure_for(code: u64) -> AuthFailure {
    match code {
        2148916227 => AuthFailure::XboxBanned,
        2148916233 => AuthFailure::XboxNoAccount,
        2148916235 => AuthFailure::XboxCountry,
        2148916236 | 2148916237 => AuthFailure::XboxAdultVerification,
        2148916238 => AuthFailure::XboxChildAccount,
        other => AuthFailure::XboxUnknown(other),
    }
}

async fn post(client: &Client, url: &str, body: serde_json::Value) -> AppResult<XboxToken> {
    let response = client
        .post(url)
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status().as_u16();
        return Err(match response.json::<XboxErrorBody>().await {
            Ok(error) => failure_for(error.x_err).into(),
            Err(_) => AppError::HttpStatus { status, url: url.to_owned() },
        });
    }
    let parsed: XboxResponse = response.json().await?;
    let user_hash = parsed
        .display_claims
        .xui
        .into_iter()
        .next()
        .map(|info| info.uhs)
        .ok_or_else(|| AppError::Internal("Xbox response has no user hash".into()))?;
    Ok(XboxToken { token: parsed.token, user_hash })
}

pub async fn authenticate(client: &Client, endpoints: &Endpoints, microsoft_access_token: &str) -> AppResult<XboxToken> {
    post(
        client,
        &endpoints.xbox_user,
        json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={microsoft_access_token}"),
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
        }),
    )
    .await
}

pub async fn authorize(client: &Client, endpoints: &Endpoints, user_token: &str) -> AppResult<XboxToken> {
    post(
        client,
        &endpoints.xsts,
        json!({
            "Properties": { "SandboxId": "RETAIL", "UserTokens": [user_token] },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
        }),
    )
    .await
}
