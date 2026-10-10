mod instances;
mod launch;
mod versions;

pub use instances::*;
pub use launch::*;
pub use versions::*;

use crate::error::{AppError, AppResult};

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> AppResult<T> + Send + 'static) -> AppResult<T> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| AppError::Internal(error.to_string()))?
}
