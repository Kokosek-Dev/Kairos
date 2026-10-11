use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AuthFailure {
    #[error("no Microsoft application ID is configured")]
    NotConfigured,
    #[error("the sign-in was declined")]
    Declined,
    #[error("the sign-in code expired")]
    Expired,
    #[error("this application is not approved to use Minecraft services")]
    AppNotApproved,
    #[error("this Microsoft account has no Xbox profile")]
    XboxNoAccount,
    #[error("this Xbox account is banned")]
    XboxBanned,
    #[error("Xbox Live is not available in the account's country or region")]
    XboxCountry,
    #[error("the account needs adult verification on the Xbox website")]
    XboxAdultVerification,
    #[error("child accounts must be added to a Microsoft family first")]
    XboxChildAccount,
    #[error("Xbox authentication failed (code {0})")]
    XboxUnknown(u64),
    #[error("this account does not own Minecraft: Java Edition")]
    GameNotOwned,
    #[error("this account owns the game but has no Minecraft profile yet")]
    ProfileMissing,
    #[error("the saved sign-in expired, sign in again")]
    SessionExpired,
    #[error("the system credential store failed: {0}")]
    Keychain(String),
}

impl AuthFailure {
    fn code(&self) -> &'static str {
        match self {
            Self::NotConfigured => "auth_not_configured",
            Self::Declined => "auth_declined",
            Self::Expired => "auth_expired",
            Self::AppNotApproved => "auth_app_not_approved",
            Self::XboxNoAccount => "auth_xbox_no_account",
            Self::XboxBanned => "auth_xbox_banned",
            Self::XboxCountry => "auth_xbox_country",
            Self::XboxAdultVerification => "auth_xbox_adult_verification",
            Self::XboxChildAccount => "auth_xbox_child_account",
            Self::XboxUnknown(_) => "auth_xbox_unknown",
            Self::GameNotOwned => "auth_game_not_owned",
            Self::ProfileMissing => "auth_profile_missing",
            Self::SessionExpired => "auth_session_expired",
            Self::Keychain(_) => "auth_keychain",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] AuthFailure),
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid data: {0}")]
    Json(#[from] serde_json::Error),
    #[error("integrity check failed for {0}")]
    Checksum(String),
    #[error("version {0} was not found")]
    VersionNotFound(String),
    #[error("download failed with HTTP {status} for {url}")]
    HttpStatus { status: u16, url: String },
    #[error("no Java {0} runtime is available for this system")]
    JavaUnavailable(u32),
    #[error("you need to sign in with a Microsoft account to play")]
    NotSignedIn,
    #[error("instance {0} was not found")]
    InstanceNotFound(String),
    #[error("{0}")]
    InvalidInput(String),
    #[error("the instance is currently running")]
    InstanceRunning,
    #[error("no compatible version was found for {0}")]
    NoCompatibleVersion(String),
    #[error("the modpack is invalid: {0}")]
    InvalidModpack(String),
    #[error("skin not found")]
    SkinNotFound,
    #[error("operation cancelled")]
    Cancelled,
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::Auth(failure) => failure.code(),
            Self::Network(_) => "network",
            Self::Io(_) => "io",
            Self::Json(_) => "invalid_data",
            Self::Checksum(_) => "checksum",
            Self::VersionNotFound(_) => "version_not_found",
            Self::HttpStatus { .. } => "http_status",
            Self::JavaUnavailable(_) => "java_unavailable",
            Self::NotSignedIn => "not_signed_in",
            Self::InstanceNotFound(_) => "instance_not_found",
            Self::InvalidInput(_) => "invalid_input",
            Self::InstanceRunning => "instance_running",
            Self::NoCompatibleVersion(_) => "no_compatible_version",
            Self::InvalidModpack(_) => "invalid_modpack",
            Self::SkinNotFound => "skin_not_found",
            Self::Cancelled => "cancelled",
            Self::Internal(_) => "internal",
        }
    }
}

#[derive(Serialize)]
struct SerializedError {
    code: &'static str,
    message: String,
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SerializedError {
            code: self.code(),
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub type AppResult<T> = Result<T, AppError>;
