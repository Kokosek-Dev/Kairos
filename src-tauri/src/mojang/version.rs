use super::platform::{arch_bits, arch_matches, os_name};
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use std::collections::HashMap;

const DEFAULT_JAVA_MAJOR: u32 = 8;

/// A version profile after `inheritsFrom` has been resolved.
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
    /// Set when the client jar belongs to the parent version (loader profiles).
    #[serde(skip)]
    pub client_jar_version: Option<String>,
}

impl VersionJson {
    pub fn java_major(&self) -> u32 {
        self.java_version
            .as_ref()
            .map_or(DEFAULT_JAVA_MAJOR, |java| java.major_version)
    }

    pub fn jar_id(&self) -> &str {
        self.client_jar_version.as_deref().unwrap_or(&self.id)
    }
}

/// A version profile as stored on disk: loader profiles only contain what they change.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawVersion {
    pub id: String,
    pub inherits_from: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub main_class: Option<String>,
    pub asset_index: Option<AssetIndexRef>,
    pub downloads: Option<VersionDownloads>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    pub java_version: Option<JavaVersion>,
    pub logging: Option<Logging>,
    pub arguments: Option<Arguments>,
    pub minecraft_arguments: Option<String>,
}

fn missing(field: &str) -> AppError {
    AppError::Internal(format!("the version profile has no {field}"))
}

fn library_key(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    match parts.len() {
        0..=2 => name.to_owned(),
        3 => format!("{}:{}", parts[0], parts[1]),
        _ => format!("{}:{}:{}", parts[0], parts[1], parts[3]),
    }
}

fn merge_arguments(parent: Option<Arguments>, child: Option<Arguments>) -> Option<Arguments> {
    match (parent, child) {
        (Some(parent), Some(child)) => Some(Arguments {
            game: [parent.game, child.game].concat(),
            jvm: [parent.jvm, child.jvm].concat(),
        }),
        (parent, child) => child.or(parent),
    }
}

impl RawVersion {
    pub fn resolve(self, parent: Option<VersionJson>) -> AppResult<VersionJson> {
        let Some(parent) = parent else {
            return Ok(VersionJson {
                id: self.id,
                kind: self.kind.ok_or_else(|| missing("type"))?,
                main_class: self.main_class.ok_or_else(|| missing("main class"))?,
                asset_index: self.asset_index.ok_or_else(|| missing("asset index"))?,
                downloads: self.downloads.ok_or_else(|| missing("downloads"))?,
                libraries: self.libraries,
                java_version: self.java_version,
                logging: self.logging,
                arguments: self.arguments,
                minecraft_arguments: self.minecraft_arguments,
                client_jar_version: None,
            });
        };

        let child_keys: Vec<String> = self.libraries.iter().map(|library| library_key(&library.name)).collect();
        let inherited = parent
            .libraries
            .iter()
            .filter(|library| !child_keys.contains(&library_key(&library.name)))
            .cloned();
        let libraries = self.libraries.iter().cloned().chain(inherited).collect();

        Ok(VersionJson {
            client_jar_version: Some(parent.jar_id().to_owned()),
            id: self.id,
            kind: self.kind.unwrap_or(parent.kind),
            main_class: self.main_class.unwrap_or(parent.main_class),
            asset_index: self.asset_index.unwrap_or(parent.asset_index),
            downloads: self.downloads.unwrap_or(parent.downloads),
            libraries,
            java_version: self.java_version.or(parent.java_version),
            logging: self.logging.or(parent.logging),
            arguments: merge_arguments(parent.arguments, self.arguments),
            minecraft_arguments: self.minecraft_arguments.or(parent.minecraft_arguments),
        })
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
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<Extract>,
    /// Maven repository base URL, used by loader profiles that list only coordinates.
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
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

/// Where a library file lives and how to fetch it. An empty URL means the file is produced
/// locally (for example by a loader installer) and cannot be downloaded.
#[derive(Debug, Clone)]
pub struct ArtifactRef {
    pub path: String,
    pub url: String,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

impl From<&LibraryArtifact> for ArtifactRef {
    fn from(artifact: &LibraryArtifact) -> Self {
        Self {
            path: artifact.path.clone(),
            url: artifact.url.clone(),
            sha1: Some(artifact.sha1.clone()),
            size: Some(artifact.size),
        }
    }
}

pub fn maven_path(coordinates: &str) -> Option<String> {
    let (coordinates, extension) = coordinates.split_once('@').unwrap_or((coordinates, "jar"));
    let mut parts = coordinates.split(':');
    let (group, artifact, version) = (parts.next()?, parts.next()?, parts.next()?);
    let classifier = parts.next().map(|classifier| format!("-{classifier}")).unwrap_or_default();
    Some(format!(
        "{}/{artifact}/{version}/{artifact}-{version}{classifier}.{extension}",
        group.replace('.', "/")
    ))
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
    /// Kairos does not enable any optional launcher feature (demo mode, quick play,
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

    pub fn main_artifact(&self) -> Option<ArtifactRef> {
        if let Some(artifact) = self.downloads.as_ref().and_then(|downloads| downloads.artifact.as_ref()) {
            return Some(artifact.into());
        }
        if self.downloads.is_some() {
            return None;
        }
        let path = maven_path(&self.name)?;
        let base = self.url.as_deref()?;
        Some(ArtifactRef {
            url: format!("{}{path}", if base.ends_with('/') { base.to_owned() } else { format!("{base}/") }),
            path,
            sha1: self.sha1.clone(),
            size: self.size,
        })
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
            r#"{"name":"a:b:1","natives":{"windows":"natives-windows-${arch}","linux":"natives-linux","osx":"natives-osx"}}"#,
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

    #[test]
    fn builds_maven_paths_with_classifiers_and_extensions() {
        assert_eq!(
            maven_path("net.fabricmc:fabric-loader:0.16.14").unwrap(),
            "net/fabricmc/fabric-loader/0.16.14/fabric-loader-0.16.14.jar"
        );
        assert_eq!(
            maven_path("org.lwjgl:lwjgl:3.3.3:natives-windows").unwrap(),
            "org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-windows.jar"
        );
        assert_eq!(maven_path("a.b:c:1.0@zip").unwrap(), "a/b/c/1.0/c-1.0.zip");
        assert!(maven_path("broken").is_none());
    }

    #[test]
    fn libraries_listed_by_coordinates_resolve_against_their_maven_url() {
        let library: Library = serde_json::from_str(
            r#"{"name":"org.ow2.asm:asm:9.7","url":"https://maven.fabricmc.net/","sha1":"abc","size":12}"#,
        )
        .unwrap();

        let artifact = library.main_artifact().unwrap();

        assert_eq!(artifact.path, "org/ow2/asm/asm/9.7/asm-9.7.jar");
        assert_eq!(artifact.url, "https://maven.fabricmc.net/org/ow2/asm/asm/9.7/asm-9.7.jar");
        assert_eq!((artifact.sha1.as_deref(), artifact.size), (Some("abc"), Some(12)));
    }

    const PARENT: &str = r#"{
        "id": "1.21.1", "type": "release", "mainClass": "net.minecraft.client.main.Main",
        "assetIndex": {"id": "17", "sha1": "a", "size": 1, "url": "u"},
        "downloads": {"client": {"sha1": "a", "size": 1, "url": "u"}},
        "javaVersion": {"majorVersion": 21},
        "libraries": [
            {"name": "org.ow2.asm:asm:9.6", "downloads": {"artifact": {"path": "p/asm-9.6.jar", "sha1": "a", "size": 1, "url": "u"}}},
            {"name": "com.mojang:logging:1.0", "downloads": {"artifact": {"path": "p/logging.jar", "sha1": "a", "size": 1, "url": "u"}}}
        ],
        "arguments": {"game": ["--username", "x"], "jvm": ["-Dparent=1"]}
    }"#;

    const CHILD: &str = r#"{
        "id": "fabric-loader-0.16.14-1.21.1", "inheritsFrom": "1.21.1", "type": "release",
        "mainClass": "net.fabricmc.loader.impl.launch.knot.KnotClient",
        "libraries": [
            {"name": "org.ow2.asm:asm:9.7", "url": "https://maven.fabricmc.net/"},
            {"name": "net.fabricmc:fabric-loader:0.16.14", "url": "https://maven.fabricmc.net/"}
        ],
        "arguments": {"game": [], "jvm": ["-DFabricMcEmu= net.minecraft.client.main.Main "]}
    }"#;

    #[test]
    fn child_profiles_inherit_from_the_parent_and_override_what_they_declare() {
        let parent = serde_json::from_str::<RawVersion>(PARENT).unwrap().resolve(None).unwrap();
        let child: RawVersion = serde_json::from_str(CHILD).unwrap();

        let merged = child.resolve(Some(parent)).unwrap();

        assert_eq!(merged.id, "fabric-loader-0.16.14-1.21.1");
        assert_eq!(merged.jar_id(), "1.21.1");
        assert_eq!(merged.main_class, "net.fabricmc.loader.impl.launch.knot.KnotClient");
        assert_eq!(merged.asset_index.id, "17");
        assert_eq!(merged.java_major(), 21);
        let names: Vec<&str> = merged.libraries.iter().map(|library| library.name.as_str()).collect();
        assert_eq!(names, ["org.ow2.asm:asm:9.7", "net.fabricmc:fabric-loader:0.16.14", "com.mojang:logging:1.0"]);
        let arguments = merged.arguments.unwrap();
        let jvm = resolve_arguments(&arguments.jvm);
        assert_eq!(jvm, ["-Dparent=1", "-DFabricMcEmu= net.minecraft.client.main.Main "]);
        assert_eq!(resolve_arguments(&arguments.game), ["--username", "x"]);
    }

    #[test]
    fn a_profile_without_a_parent_must_be_complete() {
        let child: RawVersion = serde_json::from_str(CHILD).unwrap();

        assert!(child.resolve(None).is_err());
    }
}
