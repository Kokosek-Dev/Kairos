use crate::error::{AppError, AppResult};
use crate::paths::DataPaths;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use tokio_util::sync::CancellationToken;

struct ActiveRun {
    instance_id: String,
    token: CancellationToken,
}

#[derive(Clone, Default)]
pub struct RunRegistry {
    active: Arc<Mutex<HashMap<u64, ActiveRun>>>,
    next_id: Arc<AtomicU64>,
}

impl RunRegistry {
    pub fn register(&self, instance_id: &str) -> (u64, CancellationToken) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let token = CancellationToken::new();
        let run = ActiveRun { instance_id: instance_id.to_owned(), token: token.clone() };
        self.lock().insert(id, run);
        (id, token)
    }

    pub fn is_instance_running(&self, instance_id: &str) -> bool {
        self.lock().values().any(|run| run.instance_id == instance_id)
    }

    pub fn finish(&self, run_id: u64) {
        self.lock().remove(&run_id);
    }

    pub fn stop(&self, run_id: u64) {
        if let Some(run) = self.lock().get(&run_id) {
            run.token.cancel();
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<u64, ActiveRun>> {
        self.active.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub struct AppState {
    pub client: Client,
    pub paths: DataPaths,
    pub runs: RunRegistry,
    installs: Mutex<HashMap<String, CancellationToken>>,
}

impl AppState {
    pub fn new(client: Client, paths: DataPaths) -> Self {
        Self {
            client,
            paths,
            runs: RunRegistry::default(),
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
