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
    /// Placeholder identity for development builds only. It cannot authenticate against
    /// online services and is never compiled into release builds.
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

/// Debug builds can skip Microsoft sign-in when `ZENITH_DEV_SESSION=1` is set explicitly.
pub fn development_override() -> Option<GameSession> {
    #[cfg(debug_assertions)]
    return (std::env::var("ZENITH_DEV_SESSION").as_deref() == Ok("1")).then(GameSession::development);
    #[cfg(not(debug_assertions))]
    return None;
}
