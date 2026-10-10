use crate::error::{AppError, AppResult, AuthFailure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

const KEYRING_SERVICE: &str = "zenith-launcher";

pub trait SecretStore: Send + Sync {
    fn set(&self, key: &str, secret: &[u8]) -> AppResult<()>;
    fn get(&self, key: &str) -> AppResult<Option<Vec<u8>>>;
    fn delete(&self, key: &str) -> AppResult<()>;
}

pub struct KeyringStore;

fn keychain(error: keyring::Error) -> AppError {
    AuthFailure::Keychain(error.to_string()).into()
}

impl KeyringStore {
    fn entry(key: &str) -> AppResult<keyring::Entry> {
        keyring::Entry::new(KEYRING_SERVICE, key).map_err(keychain)
    }
}

impl SecretStore for KeyringStore {
    fn set(&self, key: &str, secret: &[u8]) -> AppResult<()> {
        Self::entry(key)?.set_secret(secret).map_err(keychain)
    }

    fn get(&self, key: &str) -> AppResult<Option<Vec<u8>>> {
        match Self::entry(key)?.get_secret() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(keychain(error)),
        }
    }

    fn delete(&self, key: &str) -> AppResult<()> {
        match Self::entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(keychain(error)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub username: String,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accounts {
    pub active_id: Option<String>,
    pub accounts: Vec<Account>,
}

pub struct AccountStore {
    file: PathBuf,
    secrets: Arc<dyn SecretStore>,
}

impl AccountStore {
    pub fn new(file: PathBuf, secrets: Arc<dyn SecretStore>) -> Self {
        Self { file, secrets }
    }

    pub fn load(&self) -> AppResult<Accounts> {
        match fs::read(&self.file) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Accounts::default()),
            Err(error) => Err(error.into()),
        }
    }

    fn save(&self, accounts: &Accounts) -> AppResult<()> {
        if let Some(parent) = self.file.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = self.file.with_extension("json.tmp");
        fs::write(&temp, serde_json::to_vec_pretty(accounts)?)?;
        fs::rename(temp, &self.file)?;
        Ok(())
    }

    pub fn active(&self) -> AppResult<Option<Account>> {
        let accounts = self.load()?;
        Ok(accounts
            .active_id
            .and_then(|id| accounts.accounts.into_iter().find(|account| account.id == id)))
    }

    /// Adds the account (or refreshes its name) and makes it the active one.
    pub fn upsert(&self, account: Account, refresh_token: &str) -> AppResult<()> {
        self.secrets.set(&account.id, refresh_token.as_bytes())?;
        let mut accounts = self.load()?;
        match accounts.accounts.iter_mut().find(|existing| existing.id == account.id) {
            Some(existing) => existing.username = account.username.clone(),
            None => accounts.accounts.push(account.clone()),
        }
        accounts.active_id = Some(account.id);
        self.save(&accounts)
    }

    pub fn set_active(&self, id: &str) -> AppResult<()> {
        let mut accounts = self.load()?;
        if !accounts.accounts.iter().any(|account| account.id == id) {
            return Err(AppError::InvalidInput("unknown account".into()));
        }
        accounts.active_id = Some(id.to_owned());
        self.save(&accounts)
    }

    pub fn remove(&self, id: &str) -> AppResult<()> {
        self.secrets.delete(id)?;
        let mut accounts = self.load()?;
        accounts.accounts.retain(|account| account.id != id);
        if accounts.active_id.as_deref() == Some(id) {
            accounts.active_id = accounts.accounts.first().map(|account| account.id.clone());
        }
        self.save(&accounts)
    }

    pub fn refresh_token(&self, id: &str) -> AppResult<Option<String>> {
        Ok(self
            .secrets
            .get(id)?
            .and_then(|bytes| String::from_utf8(bytes).ok()))
    }

    pub fn save_refresh_token(&self, id: &str, refresh_token: &str) -> AppResult<()> {
        self.secrets.set(id, refresh_token.as_bytes())
    }
}

#[cfg(test)]
pub mod memory {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    pub struct MemoryStore(pub Mutex<HashMap<String, Vec<u8>>>);

    impl SecretStore for MemoryStore {
        fn set(&self, key: &str, secret: &[u8]) -> AppResult<()> {
            self.0.lock().unwrap().insert(key.to_owned(), secret.to_vec());
            Ok(())
        }

        fn get(&self, key: &str) -> AppResult<Option<Vec<u8>>> {
            Ok(self.0.lock().unwrap().get(key).cloned())
        }

        fn delete(&self, key: &str) -> AppResult<()> {
            self.0.lock().unwrap().remove(key);
            Ok(())
        }
    }
}

#[cfg(test)]
mod keyring_tests {
    use super::*;

    #[test]
    #[ignore = "uses the real operating system credential store"]
    fn the_system_keyring_stores_a_token_sized_secret() {
        let store = KeyringStore;
        let key = format!("zenith-test-{}", std::process::id());
        let secret = "M.C5_".to_owned() + &"x".repeat(2000);

        store.set(&key, secret.as_bytes()).unwrap();
        let loaded = store.get(&key).unwrap().unwrap();
        store.delete(&key).unwrap();

        assert_eq!(loaded, secret.as_bytes());
        assert!(store.get(&key).unwrap().is_none());
    }
}