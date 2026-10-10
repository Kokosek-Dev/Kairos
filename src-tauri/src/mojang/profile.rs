use super::version::{RawVersion, VersionJson};
use crate::error::{AppError, AppResult};
use crate::paths::DataPaths;
use tokio::fs;

const MAX_INHERITANCE_DEPTH: usize = 4;

/// Loads a version profile from disk and folds in everything it inherits from.
pub async fn load_resolved(paths: &DataPaths, id: &str) -> AppResult<VersionJson> {
    let mut chain: Vec<RawVersion> = Vec::new();
    let mut next = Some(id.to_owned());

    while let Some(current) = next {
        if chain.len() >= MAX_INHERITANCE_DEPTH {
            return Err(AppError::Internal(format!("{id} inherits from too many versions")));
        }
        if !is_safe_id(&current) {
            return Err(AppError::Internal(format!("{current} is not a valid version id")));
        }
        let raw: RawVersion = serde_json::from_slice(&fs::read(paths.version_json(&current)).await?)?;
        next = raw.inherits_from.clone();
        chain.push(raw);
    }

    let mut resolved = None;
    for raw in chain.into_iter().rev() {
        resolved = Some(raw.resolve(resolved)?);
    }
    resolved.ok_or_else(|| AppError::VersionNotFound(id.to_owned()))
}

/// Version ids become directory names, so anything that could escape the versions folder is refused.
pub fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id != "."
        && id != ".."
        && !id.chars().any(|character| matches!(character, '/' | '\\' | ':' | '\0') || character.is_control())
}
