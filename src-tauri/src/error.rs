use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
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
    #[cfg_attr(debug_assertions, allow(dead_code))]
    NotSignedIn,
    #[error("operation cancelled")]
    Cancelled,
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::Network(_) => "network",
            Self::Io(_) => "io",
            Self::Json(_) => "invalid_data",
            Self::Checksum(_) => "checksum",
            Self::VersionNotFound(_) => "version_not_found",
            Self::HttpStatus { .. } => "http_status",
            Self::JavaUnavailable(_) => "java_unavailable",
            Self::NotSignedIn => "not_signed_in",
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
