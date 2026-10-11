use crate::error::{AppResult, AuthFailure};

const CLIENT_ID_ENV: &str = "KAIROS_MS_CLIENT_ID";
const BUILT_IN_CLIENT_ID: Option<&str> = option_env!("KAIROS_MS_CLIENT_ID");

fn is_application_id(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}

/// The Microsoft application ID is part of the launcher itself (set once when building it, see
/// docs/AZURE_SETUP.md), so players never see it. A runtime variable overrides it for development.
pub fn client_id() -> AppResult<String> {
    std::env::var(CLIENT_ID_ENV)
        .ok()
        .or_else(|| BUILT_IN_CLIENT_ID.map(str::to_owned))
        .filter(|id| is_application_id(id))
        .ok_or_else(|| AuthFailure::NotConfigured.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_azure_application_ids() {
        assert!(is_application_id("12345678-abcd-4ef0-9abc-0123456789ab"));
        assert!(!is_application_id(""));
        assert!(!is_application_id("not-a-guid"));
        assert!(!is_application_id("12345678abcd4ef09abc0123456789ab0000"));
    }
}
