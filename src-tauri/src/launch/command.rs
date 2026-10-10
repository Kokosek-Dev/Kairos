use super::session::GameSession;
use crate::mojang::version::{resolve_arguments, VersionJson};
use crate::paths::DataPaths;
use std::collections::HashMap;
use std::path::PathBuf;

const LAUNCHER_NAME: &str = "zenith";
const LEGACY_JVM_ARGUMENTS: [&str; 5] = [
    "-Djava.library.path=${natives_directory}",
    "-Dminecraft.launcher.brand=${launcher_name}",
    "-Dminecraft.launcher.version=${launcher_version}",
    "-cp",
    "${classpath}",
];

pub struct LaunchSpec<'a> {
    pub version: &'a VersionJson,
    pub paths: &'a DataPaths,
    pub java: PathBuf,
    pub game_dir: PathBuf,
    pub game_assets: PathBuf,
    pub memory_mb: u32,
    pub extra_jvm_args: Vec<String>,
    pub session: &'a GameSession,
}

#[derive(Debug)]
pub struct LaunchCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
}

fn classpath_separator() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}

fn classpath(spec: &LaunchSpec) -> String {
    let mut entries: Vec<PathBuf> = Vec::new();
    let libraries = spec
        .version
        .libraries
        .iter()
        .filter(|library| library.applies())
        .filter_map(|library| library.downloads.as_ref()?.artifact.as_ref())
        .map(|artifact| spec.paths.libraries().join(&artifact.path));
    for path in libraries.chain([spec.paths.client_jar(&spec.version.id)]) {
        if !entries.contains(&path) {
            entries.push(path);
        }
    }
    entries
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join(classpath_separator())
}

fn variables(spec: &LaunchSpec) -> HashMap<&'static str, String> {
    let session = spec.session;
    let path = |path: PathBuf| path.to_string_lossy().into_owned();
    HashMap::from([
        ("auth_player_name", session.username.clone()),
        ("auth_uuid", session.uuid.clone()),
        ("auth_access_token", session.access_token.clone()),
        (
            "auth_session",
            format!("token:{}:{}", session.access_token, session.uuid),
        ),
        ("auth_xuid", session.xuid.clone()),
        ("clientid", session.client_id.clone()),
        ("user_type", session.user_type.clone()),
        ("user_properties", "{}".into()),
        ("version_name", spec.version.id.clone()),
        ("version_type", spec.version.kind.clone()),
        ("game_directory", path(spec.game_dir.clone())),
        ("assets_root", path(spec.paths.assets())),
        ("assets_index_name", spec.version.asset_index.id.clone()),
        ("game_assets", path(spec.game_assets.clone())),
        ("natives_directory", path(spec.paths.natives_dir(&spec.version.id))),
        ("library_directory", path(spec.paths.libraries())),
        ("classpath", classpath(spec)),
        ("classpath_separator", classpath_separator().into()),
        ("launcher_name", LAUNCHER_NAME.into()),
        ("launcher_version", env!("CARGO_PKG_VERSION").into()),
    ])
}

fn substitute(template: &str, variables: &HashMap<&'static str, String>) -> String {
    variables.iter().fold(template.to_owned(), |text, (key, value)| {
        text.replace(&format!("${{{key}}}"), value)
    })
}

pub fn build_command(spec: &LaunchSpec) -> LaunchCommand {
    let version = spec.version;
    let (jvm, game) = match &version.arguments {
        Some(arguments) => (resolve_arguments(&arguments.jvm), resolve_arguments(&arguments.game)),
        None => (
            LEGACY_JVM_ARGUMENTS.map(String::from).to_vec(),
            version
                .minecraft_arguments
                .as_deref()
                .unwrap_or_default()
                .split_whitespace()
                .map(String::from)
                .collect(),
        ),
    };

    let logging = version
        .logging
        .as_ref()
        .and_then(|logging| logging.client.as_ref())
        .map(|client| {
            let config = spec.paths.log_configs().join(&client.file.id);
            client.argument.replace("${path}", &config.to_string_lossy())
        });

    let variables = variables(spec);
    let args = [format!("-Xmx{}M", spec.memory_mb)]
        .into_iter()
        .chain(jvm.iter().map(|arg| substitute(arg, &variables)))
        .chain(logging)
        .chain(spec.extra_jvm_args.iter().cloned())
        .chain([version.main_class.clone()])
        .chain(game.iter().map(|arg| substitute(arg, &variables)))
        .collect();

    LaunchCommand {
        program: spec.java.clone(),
        args,
        working_dir: spec.game_dir.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mojang::platform::os_name;

    fn version(json: &str) -> VersionJson {
        serde_json::from_str(json).unwrap()
    }

    const MODERN: &str = r#"{
        "id": "1.21.1", "type": "release", "mainClass": "net.minecraft.client.main.Main",
        "assetIndex": {"id": "17", "sha1": "a", "size": 1, "url": "u"},
        "downloads": {"client": {"sha1": "a", "size": 1, "url": "u"}},
        "libraries": [
            {"downloads": {"artifact": {"path": "a/a.jar", "sha1": "a", "size": 1, "url": "u"}}},
            {"downloads": {"artifact": {"path": "a/a.jar", "sha1": "a", "size": 1, "url": "u"}}},
            {"downloads": {"artifact": {"path": "mac/m.jar", "sha1": "a", "size": 1, "url": "u"}},
             "rules": [{"action": "allow", "os": {"name": "osx"}}]}
        ],
        "logging": {"client": {"argument": "-Dlog4j.configurationFile=${path}",
            "file": {"id": "client-1.12.xml", "sha1": "a", "size": 1, "url": "u"}}},
        "arguments": {
            "game": ["--username", "${auth_player_name}", "--version", "${version_name}",
                {"rules": [{"action": "allow", "features": {"is_demo_user": true}}], "value": "--demo"}],
            "jvm": ["-Djava.library.path=${natives_directory}", "-cp", "${classpath}"]
        }
    }"#;

    const LEGACY: &str = r#"{
        "id": "1.0", "type": "release", "mainClass": "net.minecraft.launchwrapper.Launch",
        "assetIndex": {"id": "pre-1.6", "sha1": "a", "size": 1, "url": "u"},
        "downloads": {"client": {"sha1": "a", "size": 1, "url": "u"}},
        "minecraftArguments": "${auth_player_name} ${auth_session} --gameDir ${game_directory} --assetsDir ${game_assets}"
    }"#;

    fn build(version: &VersionJson, session: &GameSession) -> LaunchCommand {
        let paths = DataPaths::new(PathBuf::from("data"));
        build_command(&LaunchSpec {
            version,
            paths: &paths,
            java: PathBuf::from("java"),
            game_dir: PathBuf::from("game"),
            game_assets: PathBuf::from("assets-for-game"),
            memory_mb: 3072,
            extra_jvm_args: vec!["-Dextra=1".into()],
            session,
        })
    }

    fn session() -> GameSession {
        GameSession {
            username: "Steve".into(),
            uuid: "uuid-1".into(),
            access_token: "tok".into(),
            user_type: "msa".into(),
            xuid: "1".into(),
            client_id: "2".into(),
        }
    }

    #[test]
    fn modern_command_orders_memory_jvm_logging_main_and_game_arguments() {
        let command = build(&version(MODERN), &session());
        let args = &command.args;

        assert_eq!(args[0], "-Xmx3072M");
        let main = args.iter().position(|arg| arg == "net.minecraft.client.main.Main").unwrap();
        assert_eq!(&args[main + 1..], ["--username", "Steve", "--version", "1.21.1"]);
        let logging = args.iter().position(|arg| arg.starts_with("-Dlog4j.configurationFile=")).unwrap();
        assert!(args[logging].ends_with("client-1.12.xml"));
        assert!(logging < main);
        assert!(args.contains(&"-Dextra=1".to_owned()));
        assert!(!args.iter().any(|arg| arg.contains("${")));
    }

    #[test]
    fn classpath_deduplicates_filters_by_os_and_ends_with_the_client_jar() {
        let command = build(&version(MODERN), &session());
        let classpath = &command.args[command.args.iter().position(|arg| arg == "-cp").unwrap() + 1];
        let entries: Vec<&str> = classpath.split(classpath_separator()).collect();

        assert_eq!(entries.iter().filter(|entry| entry.ends_with("a.jar")).count(), 1);
        assert_eq!(entries.iter().any(|entry| entry.ends_with("m.jar")), os_name() == "osx");
        assert!(entries.last().unwrap().ends_with("1.21.1.jar"));
    }

    #[test]
    fn legacy_versions_use_default_jvm_arguments_and_split_minecraft_arguments() {
        let command = build(&version(LEGACY), &session());
        let args = &command.args;

        assert!(args.iter().any(|arg| arg.starts_with("-Djava.library.path=")));
        let main = args.iter().position(|arg| arg == "net.minecraft.launchwrapper.Launch").unwrap();
        assert_eq!(args[main + 1], "Steve");
        assert_eq!(args[main + 2], "token:tok:uuid-1");
        assert_eq!(args.last().unwrap(), "assets-for-game");
    }
}
