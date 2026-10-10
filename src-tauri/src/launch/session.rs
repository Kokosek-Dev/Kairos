use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct GameSession {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
    pub xuid: String,
    pub client_id: String,
}

#[cfg(debug_assertions)]
impl GameSession {
    /// Placeholder identity used by development builds until Microsoft sign-in exists.
    /// It cannot authenticate against online services and is compiled out of release builds.
    pub fn development() -> Self {
        Self {
            username: "ZenithDev".into(),
            uuid: "00000000-0000-0000-0000-000000000000".into(),
            access_token: "0".into(),
            user_type: "legacy".into(),
            xuid: "0".into(),
            client_id: "0".into(),
        }
    }
}

pub fn current() -> AppResult<GameSession> {
    #[cfg(debug_assertions)]
    return Ok(GameSession::development());
    #[cfg(not(debug_assertions))]
    return Err(crate::error::AppError::NotSignedIn);
}
