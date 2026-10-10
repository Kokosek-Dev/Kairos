use crate::error::{AppError, AppResult, AuthFailure};
use crate::paths::DataPaths;
use serde::{Deserialize, Serialize};
use std::fs;

const CLIENT_ID_ENV: &str = "ZENITH_MS_CLIENT_ID";
const BUILT_IN_CLIENT_ID: Option<&str> = option_env!("ZENITH_MS_CLIENT_ID");

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsFile {
    ms_client_id: Option<String>,
}

fn load(paths: &DataPaths) -> AppResult<SettingsFile> {
    match fs::read(paths.settings_file()) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(SettingsFile::default()),
        Err(error) => Err(error.into()),
    }
}

fn is_application_id(value: &str) -> bool {
    value.len() == 36
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| match index {
                8 | 13 | 18 | 23 => byte == b'-',
                _ => byte.is_ascii_hexdigit(),
            })
}

/// The application ID used for Microsoft sign-in: environment variable, then the
/// user's setting, then the value baked in at build time.
pub fn client_id(paths: &DataPaths) -> AppResult<String> {
    let from_env = std::env::var(CLIENT_ID_ENV).ok();
    let from_settings = load(paths)?.ms_client_id;
    from_env
        .or(from_settings)
        .or_else(|| BUILT_IN_CLIENT_ID.map(str::to_owned))
        .filter(|id| is_application_id(id))
        .ok_or_else(|| AuthFailure::NotConfigured.into())
}

pub fn saved_client_id(paths: &DataPaths) -> AppResult<Option<String>> {
    Ok(load(paths)?.ms_client_id)
}

pub fn set_client_id(paths: &DataPaths, value: Option<String>) -> AppResult<()> {
    let value = value.map(|value| value.trim().to_owned()).filter(|value| !value.is_empty());
    if value.as_deref().is_some_and(|value| !is_application_id(value)) {
        return Err(AppError::InvalidInput(
            "the application ID must look like 00000000-0000-0000-0000-000000000000".into(),
        ));
    }
    fs::create_dir_all(paths.root())?;
    let mut settings = load(paths)?;
    settings.ms_client_id = value;
    let temp = paths.settings_file().with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(&settings)?)?;
    fs::rename(temp, paths.settings_file())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_azure_application_ids() {
        assert!(is_application_id("12345678-abcd-4ef0-9abc-0123456789ab"));
        assert!(!is_application_id("not-a-guid"));
        assert!(!is_application_id("12345678abcd4ef09abc0123456789ab0000"));
    }

    #[test]
    fn stores_and_clears_the_application_id() {
        let root = std::env::temp_dir().join(format!("zenith-settings-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let paths = DataPaths::new(root.clone());

        assert!(set_client_id(&paths, Some("bad".into())).is_err());
        set_client_id(&paths, Some(" 12345678-abcd-4ef0-9abc-0123456789ab ".into())).unwrap();
        assert_eq!(saved_client_id(&paths).unwrap().as_deref(), Some("12345678-abcd-4ef0-9abc-0123456789ab"));
        set_client_id(&paths, None).unwrap();
        assert_eq!(saved_client_id(&paths).unwrap(), None);
        let _ = fs::remove_dir_all(root);
    }
}
