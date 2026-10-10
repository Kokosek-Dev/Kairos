use super::endpoints::Endpoints;
use crate::error::{AppError, AppResult, AuthFailure};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use reqwest::{Client, Url};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const SCOPE: &str = "XboxLive.signin offline_access";
const RANDOM_BYTES: usize = 32;

#[derive(Debug, Clone, Deserialize)]
pub struct MicrosoftTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct TokenError {
    error: String,
}

pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

fn random_token() -> AppResult<String> {
    let mut bytes = [0u8; RANDOM_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| AppError::Internal(error.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

impl Pkce {
    pub fn generate() -> AppResult<Self> {
        let verifier = random_token()?;
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Ok(Self { verifier, challenge })
    }
}

pub fn random_state() -> AppResult<String> {
    random_token()
}

pub fn authorize_url(
    endpoints: &Endpoints,
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    pkce: &Pkce,
) -> AppResult<String> {
    let mut url = Url::parse(&endpoints.authorize).map_err(|error| AppError::Internal(error.to_string()))?;
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_mode", "query")
        .append_pair("scope", SCOPE)
        .append_pair("state", state)
        .append_pair("code_challenge", &pkce.challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("prompt", "select_account");
    Ok(url.into())
}

pub async fn exchange_code(
    client: &Client,
    endpoints: &Endpoints,
    client_id: &str,
    code: &str,
    redirect_uri: &str,
    pkce: &Pkce,
) -> AppResult<MicrosoftTokens> {
    let response = client
        .post(&endpoints.token)
        .form(&[
            ("client_id", client_id),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", pkce.verifier.as_str()),
            ("scope", SCOPE),
        ])
        .send()
        .await?;
    if response.status().is_success() {
        return Ok(response.json().await?);
    }
    let status = response.status().as_u16();
    match response.json::<TokenError>().await {
        Ok(error) => Err(AppError::Internal(format!("Microsoft sign-in failed: {}", error.error))),
        Err(_) => Err(AppError::HttpStatus { status, url: endpoints.token.clone() }),
    }
}

pub async fn refresh(
    client: &Client,
    endpoints: &Endpoints,
    client_id: &str,
    refresh_token: &str,
) -> AppResult<MicrosoftTokens> {
    let response = client
        .post(&endpoints.token)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
            ("scope", SCOPE),
        ])
        .send()
        .await?;
    if response.status().is_success() {
        return Ok(response.json().await?);
    }
    let status = response.status().as_u16();
    match response.json::<TokenError>().await {
        Ok(error) if matches!(error.error.as_str(), "invalid_grant" | "interaction_required") => {
            Err(AuthFailure::SessionExpired.into())
        }
        _ => Err(AppError::HttpStatus { status, url: endpoints.token.clone() }),
    }
}
