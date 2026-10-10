use super::endpoints::Endpoints;
use crate::error::{AppError, AppResult, AuthFailure};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

const SCOPE: &str = "XboxLive.signin offline_access";
const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
const SLOW_DOWN_STEP: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MicrosoftTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct TokenError {
    error: String,
}

pub async fn request_device_code(client: &Client, endpoints: &Endpoints, client_id: &str) -> AppResult<DeviceCode> {
    let response = client
        .post(&endpoints.device_code)
        .form(&[("client_id", client_id), ("scope", SCOPE)])
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status().as_u16();
        return Err(AppError::HttpStatus { status, url: endpoints.device_code.clone() });
    }
    Ok(response.json().await?)
}

pub async fn poll_for_tokens(
    client: &Client,
    endpoints: &Endpoints,
    client_id: &str,
    code: &DeviceCode,
    cancel: &CancellationToken,
) -> AppResult<MicrosoftTokens> {
    let deadline = Instant::now() + Duration::from_secs(code.expires_in);
    let mut interval = Duration::from_secs(code.interval);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return Err(AppError::Cancelled),
            _ = tokio::time::sleep(interval) => {}
        }
        if Instant::now() > deadline {
            return Err(AuthFailure::Expired.into());
        }

        let response = client
            .post(&endpoints.token)
            .form(&[
                ("grant_type", DEVICE_GRANT),
                ("client_id", client_id),
                ("device_code", code.device_code.as_str()),
            ])
            .send()
            .await?;
        if response.status().is_success() {
            return Ok(response.json().await?);
        }

        let error: TokenError = response.json().await?;
        match error.error.as_str() {
            "authorization_pending" => {}
            "slow_down" => interval += SLOW_DOWN_STEP,
            "authorization_declined" => return Err(AuthFailure::Declined.into()),
            "expired_token" | "bad_verification_code" => return Err(AuthFailure::Expired.into()),
            other => return Err(AppError::Internal(format!("Microsoft sign-in failed: {other}"))),
        }
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
