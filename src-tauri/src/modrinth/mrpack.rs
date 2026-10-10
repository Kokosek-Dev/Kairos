use super::api::VersionFilter;
use super::content::{self, ContentEntry, ContentKind, is_safe_file_name};
use super::install::{is_allowed_download_url, pick_version, Context};
use crate::download::{download_with_progress, Checksum, DownloadItem};
use crate::error::{AppError, AppResult};
use crate::instances::model::{IconInput, Instance, LoaderKind, NewInstance};
use crate::instances::store;
use crate::tasks::blocking;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;
use zip::write::SimpleFileOptions;

const INDEX_NAME: &str = "modrinth.index.json";
const MAX_FILES: usize = 5000;
const MAX_UNPACKED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_NAME_CHARS: usize = 64;
const LOADER_KEYS: [(&str, LoaderKind); 4] = [
    ("fabric-loader", LoaderKind::Fabric),
    ("quilt-loader", LoaderKind::Quilt),
    ("forge", LoaderKind::Forge),
    ("neoforge", LoaderKind::NeoForge),
];
const OVERRIDE_DIRS: [&str; 4] = ["config", "defaultconfigs", "kubejs", "scripts"];
const OVERRIDE_FILES: [&str; 1] = ["options.txt"];

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Index {
    format_version: u32,
    game: String,
    version_id: String,
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    files: Vec<PackFile>,
    dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackFile {
    path: String,
    hashes: PackHashes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    env: Option<PackEnv>,
    downloads: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file_size: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PackHashes {
    #[serde(default)]
    sha1: String,
    #[serde(default)]
    sha512: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PackEnv {
    client: Option<String>,
    server: Option<String>,
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::InvalidModpack(message.into())
}

/// A pack path must stay inside the instance: relative, forward slashes, no `..`, no drive letters.
pub fn is_safe_pack_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.starts_with('/')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != ".." && !part.contains(':') && !part.contains('\0'))
}

fn read_index(archive: &Path) -> AppResult<Index> {
    let mut zip = zip::ZipArchive::new(File::open(archive)?).map_err(|_| invalid("this is not a .mrpack archive"))?;
    let entry = zip.by_name(INDEX_NAME).map_err(|_| invalid("modrinth.index.json is missing"))?;
    let index: Index = serde_json::from_reader(entry).map_err(|error| invalid(format!("modrinth.index.json is unreadable: {error}")))?;

    if index.format_version != 1 || index.game != "minecraft" {
        return Err(invalid("unsupported modpack format"));
    }
    if index.files.len() > MAX_FILES {
        return Err(invalid("the modpack lists too many files"));
    }
    if !index.dependencies.contains_key("minecraft") {
        return Err(invalid("the modpack does not name a Minecraft version"));
    }
    for file in &index.files {
        if !is_safe_pack_path(&file.path) {
            return Err(invalid(format!("unsafe file path {}", file.path)));
        }
        if file.downloads.is_empty() || !file.downloads.iter().all(|url| is_allowed_download_url(url)) {
            return Err(invalid(format!("{} is not downloadable from an allowed host", file.path)));
        }
        if file.hashes.sha512.is_empty() && file.hashes.sha1.is_empty() {
            return Err(invalid(format!("{} has no hash", file.path)));
        }
    }
    Ok(index)
}

fn loader_of(index: &Index) -> AppResult<(LoaderKind, Option<String>)> {
    let found: Vec<_> = LOADER_KEYS
        .iter()
        .filter_map(|(key, kind)| index.dependencies.get(*key).map(|version| (*kind, version.clone())))
        .collect();
    match found.as_slice() {
        [] => Ok((LoaderKind::Vanilla, None)),
        [(kind, version)] => Ok((*kind, Some(version.clone()))),
        _ => Err(invalid("the modpack lists more than one mod loader")),
    }
}

fn extract_overrides(archive: &Path, destination: &Path) -> AppResult<()> {
    let mut zip = zip::ZipArchive::new(File::open(archive)?).map_err(|_| invalid("this is not a .mrpack archive"))?;
    let mut unpacked = 0u64;
    for prefix in ["overrides/", "client-overrides/"] {
        for index in 0..zip.len() {
            let mut entry = zip.by_index(index).map_err(|error| invalid(error.to_string()))?;
            let Some(relative) = entry.enclosed_name().and_then(|name| name.to_str().map(str::to_owned)) else { continue };
            let relative = relative.replace('\\', "/");
            let Some(inside) = relative.strip_prefix(prefix) else { continue };
            if inside.is_empty() || !is_safe_pack_path(inside.trim_end_matches('/')) {
                continue;
            }
            let target = destination.join(inside);
            if entry.is_dir() {
                fs::create_dir_all(&target)?;
                continue;
            }
            unpacked += entry.size();
            if unpacked > MAX_UNPACKED_BYTES {
                return Err(invalid("the modpack is too large when unpacked"));
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            std::io::copy(&mut entry, &mut File::create(&target)?)?;
        }
    }
    Ok(())
}

fn content_kind_of(path: &str) -> Option<(ContentKind, &str)> {
    let (folder, name) = path.split_once('/')?;
    let kind = ContentKind::ALL.into_iter().find(|kind| kind.folder() == folder)?;
    (!name.contains('/') && is_safe_file_name(name)).then_some((kind, name))
}

fn pack_entry(file: &PackFile, kind: ContentKind, name: &str) -> ContentEntry {
    let env = |side: Option<&String>| side.cloned();
    ContentEntry {
        kind,
        file_name: name.to_owned(),
        title: name.to_owned(),
        project_id: None,
        version_id: None,
        version_number: None,
        icon_url: None,
        sha512: Some(file.hashes.sha512.clone()).filter(|hash| !hash.is_empty()),
        download_url: file.downloads.first().cloned(),
        size: file.file_size,
        client_side: env(file.env.as_ref().and_then(|env| env.client.as_ref())),
        server_side: env(file.env.as_ref().and_then(|env| env.server.as_ref())),
        dependency: false,
        installed_at: Utc::now(),
    }
}

/// Fills in project details for files that came from Modrinth so they can be updated later.
async fn identify(ctx: &Context<'_>, entries: &mut [ContentEntry]) {
    let hashes: Vec<String> = entries.iter().filter_map(|entry| entry.sha512.clone()).collect();
    let Ok(versions) = ctx.api.versions_from_hashes(&hashes, "sha512").await else { return };
    let project_ids: Vec<String> = versions.values().map(|version| version.project_id.clone()).collect();
    let Ok(projects) = ctx.api.projects(&project_ids).await else { return };
    let projects: HashMap<_, _> = projects.into_iter().map(|project| (project.id.clone(), project)).collect();

    for entry in entries {
        let Some(version) = entry.sha512.as_ref().and_then(|hash| versions.get(hash)) else { continue };
        entry.project_id = Some(version.project_id.clone());
        entry.version_id = Some(version.id.clone());
        entry.version_number = Some(version.version_number.clone());
        if let Some(project) = projects.get(&version.project_id) {
            entry.title = project.title.clone();
            entry.icon_url = project.icon_url.clone();
            entry.client_side = project.client_side.clone().or(entry.client_side.take());
            entry.server_side = project.server_side.clone().or(entry.server_side.take());
        }
    }
}

pub async fn import(
    ctx: &Context<'_>,
    archive: &Path,
    cancel: &CancellationToken,
    mut on_progress: impl FnMut(u64, u64),
) -> AppResult<Instance> {
    let archive_for_task = archive.to_path_buf();
    let index = blocking(move || read_index(&archive_for_task)).await?;
    let (loader, loader_version) = loader_of(&index)?;
    let game_version = index.dependencies["minecraft"].clone();

    let name: String = index.name.trim().chars().take(MAX_NAME_CHARS).collect();
    let new = NewInstance {
        name: if name.is_empty() { "Imported modpack".into() } else { name },
        game_version,
        icon: IconInput::Preset { key: "cube".into() },
        loader,
        loader_version,
    };
    let paths = ctx.paths.clone();
    let instance = blocking(move || store::create(&paths, new)).await?;

    match populate(ctx, &instance, archive, index, cancel, &mut on_progress).await {
        Ok(()) => Ok(instance),
        Err(error) => {
            let (paths, id) = (ctx.paths.clone(), instance.id.clone());
            let _ = blocking(move || store::delete(&paths, &id)).await;
            Err(error)
        }
    }
}

async fn populate(
    ctx: &Context<'_>,
    instance: &Instance,
    archive: &Path,
    index: Index,
    cancel: &CancellationToken,
    on_progress: &mut impl FnMut(u64, u64),
) -> AppResult<()> {
    let game_dir = ctx.paths.instance_game_dir(&instance.id);
    let wanted: Vec<&PackFile> = index
        .files
        .iter()
        .filter(|file| file.env.as_ref().and_then(|env| env.client.as_deref()) != Some("unsupported"))
        .collect();

    let items: Vec<DownloadItem> = wanted
        .iter()
        .map(|file| DownloadItem {
            url: file.downloads[0].clone(),
            path: file.path.split('/').fold(game_dir.clone(), |path, part| path.join(part)),
            checksum: Some(if file.hashes.sha512.is_empty() {
                Checksum::Sha1(file.hashes.sha1.clone())
            } else {
                Checksum::Sha512(file.hashes.sha512.clone())
            }),
            size: file.file_size,
        })
        .collect();
    let total: u64 = items.iter().filter_map(|item| item.size).sum();
    download_with_progress(ctx.api.download_client(), items, cancel, |counters| on_progress(counters.bytes(), total)).await?;

    let (archive, destination) = (archive.to_path_buf(), game_dir);
    blocking(move || extract_overrides(&archive, &destination)).await?;

    let mut entries: Vec<ContentEntry> = wanted
        .iter()
        .filter_map(|file| content_kind_of(&file.path).map(|(kind, name)| pack_entry(file, kind, name)))
        .collect();
    identify(ctx, &mut entries).await;
    let (paths, id) = (ctx.paths.clone(), instance.id.clone());
    blocking(move || content::save_entries(&paths, &id, entries)).await
}

/// Installs a modpack published on Modrinth as a new instance.
pub async fn install_modpack(
    ctx: &Context<'_>,
    project_id: &str,
    pinned_version: Option<&str>,
    cancel: &CancellationToken,
    on_progress: impl FnMut(u64, u64),
) -> AppResult<Instance> {
    let project = ctx.api.project(project_id).await?;
    if project.project_type != "modpack" {
        return Err(AppError::InvalidInput("this project is not a modpack".into()));
    }
    let version = match pinned_version {
        Some(id) => ctx.api.version(id).await?,
        None => pick_version(ctx.api.versions(&project.id, &VersionFilter { loaders: vec![], game_versions: vec![] }).await?)
            .ok_or_else(|| AppError::NoCompatibleVersion(project.title.clone()))?,
    };
    let file = version.primary_file().ok_or_else(|| AppError::NoCompatibleVersion(project.title.clone()))?;
    if !is_safe_file_name(&file.filename) || !is_allowed_download_url(&file.url) {
        return Err(invalid("the modpack file cannot be downloaded safely"));
    }

    let archive = ctx.paths.installers().join("modpacks").join(&file.filename);
    let item = DownloadItem {
        url: file.url.clone(),
        path: archive.clone(),
        checksum: Some(Checksum::Sha512(file.hashes.sha512.clone())).filter(|_| !file.hashes.sha512.is_empty()),
        size: Some(file.size).filter(|size| *size > 0),
    };
    download_with_progress(ctx.api.download_client(), vec![item], cancel, |_| {}).await?;
    import(ctx, &archive, cancel, on_progress).await
}

fn sha1_of(path: &Path) -> AppResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn collect_files(root: &Path, relative: &str, found: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root.join(relative)) else { return };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else { continue };
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = format!("{relative}/{name}");
        if kind.is_dir() {
            collect_files(root, &path, found);
        } else if kind.is_file() {
            found.push(path);
        }
    }
}

pub struct ExportOptions {
    pub name: String,
    pub version: String,
}

/// Writes the instance as a `.mrpack`: files from Modrinth are referenced by URL and hash,
/// everything else that matters (configs, hand-made content) is bundled as overrides.
pub fn export(paths: &crate::paths::DataPaths, instance: &Instance, destination: &Path, options: &ExportOptions) -> AppResult<()> {
    let game_dir = paths.instance_game_dir(&instance.id);
    let entries = content::load_entries(paths, &instance.id)?;

    let mut files = Vec::new();
    let mut referenced: Vec<String> = Vec::new();
    for entry in entries.iter().filter(|entry| entry.sha512.is_some() && entry.download_url.is_some()) {
        let path = format!("{}/{}", entry.kind.folder(), entry.file_name);
        let local = game_dir.join(entry.kind.folder()).join(&entry.file_name);
        if !local.is_file() {
            continue;
        }
        files.push(PackFile {
            path: path.clone(),
            hashes: PackHashes { sha1: sha1_of(&local)?, sha512: entry.sha512.clone().unwrap_or_default() },
            env: Some(PackEnv {
                client: Some(entry.client_side.clone().unwrap_or_else(|| "required".into())),
                server: Some(entry.server_side.clone().unwrap_or_else(|| "required".into())),
            }),
            downloads: entry.download_url.iter().cloned().collect(),
            file_size: Some(fs::metadata(&local)?.len()),
        });
        referenced.push(path);
    }

    let mut dependencies = BTreeMap::from([("minecraft".to_owned(), instance.game_version.clone())]);
    if let (Some((key, _)), Some(version)) = (
        LOADER_KEYS.iter().find(|(_, kind)| *kind == instance.loader),
        instance.loader_version.as_ref(),
    ) {
        dependencies.insert((*key).to_owned(), version.clone());
    }

    let index = Index {
        format_version: 1,
        game: "minecraft".into(),
        version_id: options.version.clone(),
        name: options.name.clone(),
        summary: None,
        files,
        dependencies,
    };

    let mut overrides = Vec::new();
    for directory in OVERRIDE_DIRS {
        collect_files(&game_dir, directory, &mut overrides);
    }
    for file in OVERRIDE_FILES {
        if game_dir.join(file).is_file() {
            overrides.push(file.to_owned());
        }
    }
    for kind in ContentKind::ALL {
        collect_files(&game_dir, kind.folder(), &mut overrides);
    }
    overrides.retain(|path| !referenced.contains(path) && !path.ends_with(".disabled") && is_safe_pack_path(path));

    let mut zip = zip::ZipWriter::new(File::create(destination)?);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(INDEX_NAME, options).map_err(|error| AppError::Internal(error.to_string()))?;
    zip.write_all(&serde_json::to_vec_pretty(&index)?)?;
    for path in overrides {
        zip.start_file(format!("overrides/{path}"), options).map_err(|error| AppError::Internal(error.to_string()))?;
        let mut source = File::open(path.split('/').fold(game_dir.clone(), |base: PathBuf, part| base.join(part)))?;
        std::io::copy(&mut source, &mut zip)?;
    }
    zip.finish().map_err(|error| AppError::Internal(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modrinth::api::ModrinthApi;
    use crate::paths::DataPaths;
    use crate::test_support::start;
    use sha2::Sha512;

    fn sha512(text: &str) -> String {
        hex::encode(Sha512::digest(text.as_bytes()))
    }

    fn scratch(name: &str) -> DataPaths {
        let paths = DataPaths::new(std::env::temp_dir().join(format!("zenith-mrpack-{name}-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        fs::create_dir_all(paths.root()).unwrap();
        paths
    }

    fn build_pack(path: &Path, index: &str, extra: &[(&str, &str)]) {
        let mut zip = zip::ZipWriter::new(File::create(path).unwrap());
        let options = SimpleFileOptions::default();
        zip.start_file(INDEX_NAME, options).unwrap();
        zip.write_all(index.as_bytes()).unwrap();
        for (name, body) in extra {
            zip.start_file(*name, options).unwrap();
            zip.write_all(body.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }

    fn index_json(base: &str, files: &str) -> String {
        format!(
            r#"{{"formatVersion":1,"game":"minecraft","versionId":"1.2.3","name":"My Pack",
            "dependencies":{{"minecraft":"1.21.1","fabric-loader":"0.16.14"}},"files":[{files}]}}"#
        )
        .replace("{base}", base)
    }

    fn file_json(path: &str, content: &str, url: &str, env: &str) -> String {
        format!(
            r#"{{"path":"{path}","hashes":{{"sha1":"{}","sha512":"{}"}},"downloads":["{url}"],"fileSize":{}{env}}}"#,
            hex::encode(Sha1::digest(content.as_bytes())),
            sha512(content),
            content.len()
        )
    }

    async fn run_import(paths: &DataPaths, mock_base: &str, archive: &Path) -> AppResult<Instance> {
        let api = ModrinthApi::with_base(reqwest::Client::new(), mock_base, None);
        let context = Context { api: &api, paths };
        import(&context, archive, &CancellationToken::new(), |_, _| {}).await
    }

    #[test]
    fn validates_pack_paths() {
        for good in ["mods/a.jar", "config/sub/x.toml", "options.txt"] {
            assert!(is_safe_pack_path(good), "{good}");
        }
        for bad in ["../a", "mods/../../a", "/etc/passwd", "C:/x", "a\\b", "", "mods//a", "mods/./a"] {
            assert!(!is_safe_pack_path(bad), "{bad}");
        }
    }

    #[tokio::test]
    async fn imports_a_pack_downloading_files_and_unpacking_overrides() {
        let mock = start(vec![
            ("/files/a.jar", vec![(200, "mod-a".to_owned())]),
            ("/files/b.jar", vec![(200, "mod-b".to_owned())]),
            ("/version_files", vec![(404, "{}".to_owned())]),
        ])
        .await;
        let paths = scratch("import");
        let files = [
            file_json("mods/a.jar", "mod-a", &format!("{}/files/a.jar", mock.base), ""),
            file_json("mods/b.jar", "mod-b", &format!("{}/files/b.jar", mock.base), r#","env":{"client":"unsupported","server":"required"}"#),
        ]
        .join(",");
        let archive = paths.root().join("pack.mrpack");
        build_pack(
            &archive,
            &index_json(&mock.base, &files),
            &[("overrides/config/a.toml", "cfg=1"), ("client-overrides/options.txt", "fov:90"), ("server-overrides/server.cfg", "no")],
        );

        let instance = run_import(&paths, &mock.base, &archive).await.unwrap();

        assert_eq!((instance.name.as_str(), instance.game_version.as_str()), ("My Pack", "1.21.1"));
        assert_eq!((instance.loader, instance.loader_version.as_deref()), (LoaderKind::Fabric, Some("0.16.14")));
        let game = paths.instance_game_dir(&instance.id);
        assert_eq!(fs::read_to_string(game.join("mods/a.jar")).unwrap(), "mod-a");
        assert!(!game.join("mods/b.jar").exists(), "client-unsupported files must be skipped");
        assert_eq!(fs::read_to_string(game.join("config/a.toml")).unwrap(), "cfg=1");
        assert_eq!(fs::read_to_string(game.join("options.txt")).unwrap(), "fov:90");
        assert!(!game.join("server.cfg").exists());
        let entries = content::load_entries(&paths, &instance.id).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_name, "a.jar");
        let _ = fs::remove_dir_all(paths.root());
    }

    #[tokio::test]
    async fn rejects_unsafe_packs_before_creating_anything() {
        let paths = scratch("unsafe");
        let base = "http://127.0.0.1:1";
        let cases = [
            ("traversal", file_json("../evil.jar", "x", &format!("{base}/x"), "")),
            ("absolute", file_json("/evil.jar", "x", &format!("{base}/x"), "")),
            ("host", file_json("mods/a.jar", "x", "https://evil.example/a.jar", "")),
        ];
        for (name, file) in cases {
            let archive = paths.root().join(format!("{name}.mrpack"));
            build_pack(&archive, &index_json(base, &file), &[]);

            let result = run_import(&paths, base, &archive).await;

            assert!(matches!(result, Err(AppError::InvalidModpack(_))), "{name}");
        }
        assert!(store::list(&paths).unwrap().is_empty());

        let not_a_pack = paths.root().join("plain.zip");
        fs::write(&not_a_pack, "hello").unwrap();
        assert!(matches!(run_import(&paths, base, &not_a_pack).await, Err(AppError::InvalidModpack(_))));
        let _ = fs::remove_dir_all(paths.root());
    }

    #[tokio::test]
    async fn a_failed_download_removes_the_half_created_instance() {
        let mock = start(vec![("/files/a.jar", vec![(200, "tampered".to_owned())])]).await;
        let paths = scratch("rollback");
        let file = file_json("mods/a.jar", "mod-a", &format!("{}/files/a.jar", mock.base), "");
        let archive = paths.root().join("pack.mrpack");
        build_pack(&archive, &index_json(&mock.base, &file), &[]);

        let result = run_import(&paths, &mock.base, &archive).await;

        assert!(result.is_err());
        assert!(store::list(&paths).unwrap().is_empty());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn override_extraction_ignores_entries_that_escape_the_instance() {
        let paths = scratch("zipslip");
        let archive = paths.root().join("pack.mrpack");
        build_pack(&archive, &index_json("x", ""), &[("overrides/../../evil.txt", "evil"), ("overrides/ok.txt", "ok")]);
        let target = paths.root().join("game");
        fs::create_dir_all(&target).unwrap();

        extract_overrides(&archive, &target).unwrap();

        assert_eq!(fs::read_to_string(target.join("ok.txt")).unwrap(), "ok");
        assert!(!paths.root().join("evil.txt").exists());
        assert!(!paths.root().parent().unwrap().join("evil.txt").exists());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[tokio::test]
    async fn exports_and_reimports_an_instance() {
        let mock = start(vec![
            ("/files/a.jar", vec![(200, "mod-a".to_owned())]),
            ("/version_files", vec![(404, "{}".to_owned())]),
        ])
        .await;
        let paths = scratch("roundtrip");
        let new = NewInstance {
            name: "Source".into(),
            game_version: "1.21.1".into(),
            icon: IconInput::Preset { key: "cube".into() },
            loader: LoaderKind::Fabric,
            loader_version: Some("0.16.14".into()),
        };
        let source = store::create(&paths, new).unwrap();
        let game = paths.instance_game_dir(&source.id);
        fs::create_dir_all(game.join("mods")).unwrap();
        fs::create_dir_all(game.join("config")).unwrap();
        fs::create_dir_all(game.join("saves/World")).unwrap();
        fs::write(game.join("mods/a.jar"), "mod-a").unwrap();
        fs::write(game.join("mods/hand-made.jar"), "mine").unwrap();
        fs::write(game.join("config/a.toml"), "cfg=1").unwrap();
        fs::write(game.join("saves/World/level.dat"), "world").unwrap();
        let entry = ContentEntry {
            kind: ContentKind::Mod,
            file_name: "a.jar".into(),
            title: "A".into(),
            project_id: Some("A".into()),
            version_id: Some("v".into()),
            version_number: Some("1".into()),
            icon_url: None,
            sha512: Some(sha512("mod-a")),
            download_url: Some(format!("{}/files/a.jar", mock.base)),
            size: Some(5),
            client_side: Some("required".into()),
            server_side: Some("unsupported".into()),
            dependency: false,
            installed_at: Utc::now(),
        };
        content::save_entries(&paths, &source.id, vec![entry]).unwrap();
        let archive = paths.root().join("out.mrpack");

        export(&paths, &source, &archive, &ExportOptions { name: "Shared".into(), version: "1.0".into() }).unwrap();

        let names: Vec<String> = {
            let mut zip = zip::ZipArchive::new(File::open(&archive).unwrap()).unwrap();
            (0..zip.len()).map(|index| zip.by_index(index).unwrap().name().unwrap().into_owned()).collect()
        };
        assert!(names.contains(&"overrides/config/a.toml".to_owned()));
        assert!(names.contains(&"overrides/mods/hand-made.jar".to_owned()), "untracked content is bundled");
        assert!(!names.iter().any(|name| name.contains("saves") || name.contains("mods/a.jar")));

        let copy = run_import(&paths, &mock.base, &archive).await.unwrap();

        assert_eq!((copy.name.as_str(), copy.loader, copy.loader_version.as_deref()), ("Shared", LoaderKind::Fabric, Some("0.16.14")));
        let copied = paths.instance_game_dir(&copy.id);
        assert_eq!(fs::read_to_string(copied.join("mods/a.jar")).unwrap(), "mod-a");
        assert_eq!(fs::read_to_string(copied.join("mods/hand-made.jar")).unwrap(), "mine");
        assert_eq!(fs::read_to_string(copied.join("config/a.toml")).unwrap(), "cfg=1");
        assert!(!copied.join("saves").exists());
        let _ = fs::remove_dir_all(paths.root());
    }

    #[tokio::test]
    #[ignore = "installs a real modpack from Modrinth (tens of megabytes)"]
    async fn installs_a_real_modpack_from_modrinth() {
        let paths = scratch("live-modpack");
        let api = ModrinthApi::new(reqwest::Client::new(), None);
        let context = Context { api: &api, paths: &paths };

        let instance = install_modpack(&context, "fabulously-optimized", None, &CancellationToken::new(), |_, _| {})
            .await
            .unwrap();

        println!("installed {} ({:?} {:?}) for {}", instance.name, instance.loader, instance.loader_version, instance.game_version);
        let entries = content::load_entries(&paths, &instance.id).unwrap();
        let identified = entries.iter().filter(|entry| entry.project_id.is_some()).count();
        println!("{} content files, {} identified on Modrinth", entries.len(), identified);
        assert!(entries.len() > 10);
        assert!(identified * 2 > entries.len(), "most files should be recognised by hash");
        assert_ne!(instance.loader, LoaderKind::Vanilla);
        let _ = fs::remove_dir_all(paths.root());
    }}
