use crate::error::AppResult;
use crate::paths::DataPaths;
use serde::{Deserialize, Serialize};
use std::fs;

/// The oldest sessions are dropped beyond this, which keeps years of playing in a small file.
const MAX_SESSIONS: usize = 5000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub instance_id: String,
    /// Unix time in seconds when the game window opened.
    pub started_at: i64,
    pub duration_secs: u64,
}

pub fn load(paths: &DataPaths) -> AppResult<Vec<Session>> {
    match fs::read(paths.sessions_file()) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes).unwrap_or_default()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

pub fn record(paths: &DataPaths, session: Session) -> AppResult<()> {
    let mut sessions = load(paths)?;
    sessions.push(session);
    if sessions.len() > MAX_SESSIONS {
        sessions.drain(..sessions.len() - MAX_SESSIONS);
    }
    let target = paths.sessions_file();
    let temp = target.with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec(&sessions)?)?;
    fs::rename(temp, target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(name: &str) -> DataPaths {
        let root: PathBuf = std::env::temp_dir().join(format!("kairos-journal-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        DataPaths::new(root)
    }

    fn session(started_at: i64) -> Session {
        Session { instance_id: "pack".into(), started_at, duration_secs: 600 }
    }

    #[test]
    fn a_missing_or_damaged_file_reads_as_no_sessions() {
        let paths = scratch("empty");
        assert!(load(&paths).unwrap().is_empty());

        fs::write(paths.sessions_file(), b"not json").unwrap();
        assert!(load(&paths).unwrap().is_empty());
    }

    #[test]
    fn sessions_are_kept_in_order() {
        let paths = scratch("order");
        record(&paths, session(10)).unwrap();
        record(&paths, session(20)).unwrap();

        let all = load(&paths).unwrap();
        assert_eq!(all.iter().map(|entry| entry.started_at).collect::<Vec<_>>(), vec![10, 20]);
    }

    #[test]
    fn only_the_newest_sessions_are_kept() {
        let paths = scratch("trim");
        let all: Vec<Session> = (0..MAX_SESSIONS as i64 + 3).map(session).collect();
        fs::write(paths.sessions_file(), serde_json::to_vec(&all).unwrap()).unwrap();

        record(&paths, session(99_999)).unwrap();

        let kept = load(&paths).unwrap();
        assert_eq!(kept.len(), MAX_SESSIONS);
        assert_eq!(kept.last().unwrap().started_at, 99_999);
        assert_eq!(kept.first().unwrap().started_at, 4);
    }
}
