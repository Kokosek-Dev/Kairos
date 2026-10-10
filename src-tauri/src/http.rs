use crate::error::AppResult;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;

const USER_AGENT: &str = concat!("voidlight/", env!("CARGO_PKG_VERSION"));

pub fn build_client() -> AppResult<Client> {
    Ok(Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(15))
        .build()?)
}

pub async fn get_json<T: DeserializeOwned>(client: &Client, url: &str) -> AppResult<T> {
    Ok(client.get(url).send().await?.error_for_status()?.json().await?)
}
