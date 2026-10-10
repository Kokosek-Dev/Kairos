use crate::error::{AppError, AppResult};
use crate::paths::DataPaths;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};
use tokio_util::sync::CancellationToken;

pub struct AppState {
    pub client: Client,
    pub paths: DataPaths,
    installs: Mutex<HashMap<String, CancellationToken>>,
}

impl AppState {
    pub fn new(client: Client, paths: DataPaths) -> Self {
        Self {
            client,
            paths,
            installs: Mutex::default(),
        }
    }

    fn installs(&self) -> MutexGuard<'_, HashMap<String, CancellationToken>> {
        self.installs.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn register_install(&self, version_id: &str, token: CancellationToken) -> AppResult<()> {
        let mut installs = self.installs();
        if installs.contains_key(version_id) {
            return Err(AppError::Internal(format!("{version_id} is already being installed")));
        }
        installs.insert(version_id.to_owned(), token);
        Ok(())
    }

    pub fn finish_install(&self, version_id: &str) {
        self.installs().remove(version_id);
    }

    pub fn cancel_install(&self, version_id: &str) {
        if let Some(token) = self.installs().get(version_id) {
            token.cancel();
        }
    }
}
