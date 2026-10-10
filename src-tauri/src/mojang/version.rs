use super::platform::{arch_bits, arch_matches, os_name};
use serde::Deserialize;
use std::collections::HashMap;

const DEFAULT_JAVA_MAJOR: u32 = 8;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionJson {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub main_class: String,
    pub asset_index: AssetIndexRef,
    pub downloads: VersionDownloads,
    #[serde(default)]
    pub libraries: Vec<Library>,
    pub java_version: Option<JavaVersion>,
    pub logging: Option<Logging>,
    pub arguments: Option<Arguments>,
    pub minecraft_arguments: Option<String>,
}

impl VersionJson {
    pub fn java_major(&self) -> u32 {
        self.java_version
            .as_ref()
            .map_or(DEFAULT_JAVA_MAJOR, |java| java.major_version)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionDownloads {
    pub client: FileRef,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub major_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileRef {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logging {
    pub client: Option<LoggingClient>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingClient {
    pub argument: String,
    pub file: LoggingFile,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Plain(String),
    Conditional { rules: Vec<Rule>, value: ArgumentValue },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    One(String),
    Many(Vec<String>),
}

pub fn resolve_arguments(arguments: &[Argument]) -> Vec<String> {
    let mut resolved = Vec::new();
    for argument in arguments {
        match argument {
            Argument::Plain(value) => resolved.push(value.clone()),
            Argument::Conditional { rules, value } if rules_allow(Some(rules)) => match value {
                ArgumentValue::One(value) => resolved.push(value.clone()),
                ArgumentValue::Many(values) => resolved.extend(values.iter().cloned()),
            },
            Argument::Conditional { .. } => {}
        }
    }
    resolved
}

#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<Extract>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Extract {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    pub classifiers: Option<HashMap<String, LibraryArtifact>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryArtifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub action: RuleAction,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

impl Rule {
    /// Rules constrained by an OS version or by launcher features never match:
    /// Zenith does not enable any optional launcher feature (demo mode, quick play,
    /// custom resolution) and the OS-version rules only patch old Java 8 builds.
    fn matches(&self) -> bool {
        let os_matches = self.os.as_ref().is_none_or(|os| {
            os.version.is_none()
                && os.name.as_deref().is_none_or(|name| name == os_name())
                && os.arch.as_deref().is_none_or(arch_matches)
        });
        let features_match = self.features.as_ref().is_none_or(HashMap::is_empty);
        os_matches && features_match
    }
}

pub fn rules_allow(rules: Option<&[Rule]>) -> bool {
    let Some(rules) = rules.filter(|rules| !rules.is_empty()) else {
        return true;
    };
    rules
        .iter()
        .filter(|rule| rule.matches())
        .last()
        .is_some_and(|rule| rule.action == RuleAction::Allow)
}

impl Library {
    pub fn applies(&self) -> bool {
        rules_allow(self.rules.as_deref())
    }

    pub fn native_classifier(&self) -> Option<String> {
        self.natives
            .as_ref()?
            .get(os_name())
            .map(|classifier| classifier.replace("${arch}", arch_bits()))
    }

    pub fn native_artifact(&self) -> Option<&LibraryArtifact> {
        let key = self.native_classifier()?;
        self.downloads.as_ref()?.classifiers.as_ref()?.get(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(json: &str) -> Vec<Rule> {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn no_rules_means_allowed() {
        assert!(rules_allow(None));
        assert!(rules_allow(Some(&[])));
    }

    #[test]
    fn os_specific_allow_rule_filters_by_platform() {
        let osx_only = rules(r#"[{"action":"allow","os":{"name":"osx"}}]"#);
        assert_eq!(rules_allow(Some(&osx_only)), os_name() == "osx");
    }

    #[test]
    fn last_matching_rule_wins() {
        let rules = rules(r#"[{"action":"allow"},{"action":"disallow","os":{"name":"windows"}}]"#);
        assert_eq!(rules_allow(Some(&rules)), os_name() != "windows");
    }

    #[test]
    fn feature_rules_do_not_apply_to_libraries() {
        let rules = rules(r#"[{"action":"allow","features":{"is_demo_user":true}}]"#);
        assert!(!rules_allow(Some(&rules)));
    }

    #[test]
    fn os_version_rules_never_match() {
        let rules = rules(r#"[{"action":"allow","os":{"name":"windows","version":"^10\\."}}]"#);
        assert!(!rules_allow(Some(&rules)));
    }

    #[test]
    fn native_classifier_substitutes_arch() {
        let library: Library = serde_json::from_str(
            r#"{"natives":{"windows":"natives-windows-${arch}","linux":"natives-linux","osx":"natives-osx"}}"#,
        )
        .unwrap();
        let classifier = library.native_classifier().unwrap();
        assert!(!classifier.contains("${arch}"));
    }

    #[test]
    fn resolves_plain_and_conditional_arguments() {
        let arguments: Vec<Argument> = serde_json::from_str(
            r#"[
                "--username", "${auth_player_name}",
                {"rules":[{"action":"allow","features":{"is_demo_user":true}}],"value":"--demo"},
                {"rules":[{"action":"allow","os":{"name":"windows"}}],"value":["-Dwin=1","-Dwin=2"]},
                {"rules":[{"action":"allow","os":{"name":"osx"}}],"value":"-XstartOnFirstThread"}
            ]"#,
        )
        .unwrap();

        let resolved = resolve_arguments(&arguments);

        assert_eq!(&resolved[..2], ["--username", "${auth_player_name}"]);
        assert!(!resolved.contains(&"--demo".to_owned()));
        assert_eq!(resolved.contains(&"-Dwin=1".to_owned()), os_name() == "windows");
        assert_eq!(resolved.contains(&"-XstartOnFirstThread".to_owned()), os_name() == "osx");
    }
}
