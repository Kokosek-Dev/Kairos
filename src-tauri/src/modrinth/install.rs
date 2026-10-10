use super::api::{ModrinthApi, VersionFilter};
use super::content::{self, is_safe_file_name, ContentEntry, ContentKind};
use super::types::{DependencyType, Project, Version, VersionFile};
use crate::download::{download_with_progress, Checksum, DownloadItem};
use crate::error::{AppError, AppResult};
use crate::instances::model::{Instance, LoaderKind};
use crate::paths::DataPaths;
use crate::tasks::blocking;
use chrono::Utc;
use reqwest::Url;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tokio_util::sync::CancellationToken;

const MAX_PLANNED_FILES: usize = 64;
const ALLOWED_DOWNLOAD_HOSTS: [&str; 4] = ["cdn.modrinth.com", "github.com", "raw.githubusercontent.com", "gitlab.com"];

pub struct Context<'a> {
    pub api: &'a ModrinthApi,
    pub paths: &'a DataPaths,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallReport {
    pub installed: Vec<ContentEntry>,
    pub missing_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub kind: ContentKind,
    pub file_name: String,
    pub title: String,
    pub project_id: String,
    pub current_version: Option<String>,
    pub new_version: String,
    pub new_version_id: String,
}

/// Downloads may only come from hosts the Modrinth modpack format allows.
pub fn is_allowed_download_url(url: &str) -> bool {
    let Ok(url) = Url::parse(url) else { return false };
    let host = url.host_str().unwrap_or_default();
    if cfg!(test) && url.scheme() == "http" && host == "127.0.0.1" {
        return true;
    }
    url.scheme() == "https" && ALLOWED_DOWNLOAD_HOSTS.contains(&host)
}

pub fn version_filter(instance: &Instance, kind: ContentKind) -> AppResult<VersionFilter> {
    let loaders: Vec<&str> = match kind {
        ContentKind::Mod => match instance.loader {
            LoaderKind::Vanilla => return Err(AppError::InvalidInput("choose a mod loader for this instance first".into())),
            LoaderKind::Fabric => vec!["fabric"],
            LoaderKind::Quilt => vec!["quilt", "fabric"],
            LoaderKind::Forge => vec!["forge"],
            LoaderKind::NeoForge => vec!["neoforge"],
        },
        ContentKind::ResourcePack => vec!["minecraft"],
        ContentKind::Shader => vec!["iris", "optifine", "canvas", "vanilla"],
    };
    Ok(VersionFilter {
        loaders: loaders.into_iter().map(str::to_owned).collect(),
        game_versions: vec![instance.game_version.clone()],
    })
}

/// Prefers the newest stable release and falls back to the newest beta or alpha.
pub fn pick_version(versions: Vec<Version>) -> Option<Version> {
    let release = versions.iter().position(|version| version.version_type == "release");
    let index = release.unwrap_or(0);
    versions.into_iter().nth(index)
}

struct Planned {
    version: Version,
    dependency: bool,
}

async fn plan_installs(
    ctx: &Context<'_>,
    root: Version,
    kind: ContentKind,
    filter: &VersionFilter,
    already_installed: &HashSet<String>,
) -> AppResult<(Vec<Planned>, Vec<String>)> {
    let mut seen: HashSet<String> = HashSet::from([root.project_id.clone()]);
    let mut plan = vec![Planned { version: root, dependency: false }];
    let mut missing = Vec::new();
    if kind != ContentKind::Mod {
        return Ok((plan, missing));
    }

    let mut cursor = 0;
    while cursor < plan.len() {
        let dependencies = plan[cursor].version.dependencies.clone();
        cursor += 1;
        for dependency in dependencies.into_iter().filter(|dependency| dependency.dependency_type == DependencyType::Required) {
            let version = match (&dependency.version_id, &dependency.project_id) {
                (Some(version_id), _) => Some(ctx.api.version(version_id).await?),
                (None, Some(project_id)) if seen.contains(project_id) || already_installed.contains(project_id) => continue,
                (None, Some(project_id)) => pick_version(ctx.api.versions(project_id, filter).await?),
                (None, None) => continue,
            };
            let Some(version) = version else {
                missing.push(dependency.project_id.unwrap_or_default());
                continue;
            };
            if seen.contains(&version.project_id) || already_installed.contains(&version.project_id) {
                continue;
            }
            if plan.len() >= MAX_PLANNED_FILES {
                return Err(AppError::InvalidInput("this mod pulls in too many dependencies".into()));
            }
            seen.insert(version.project_id.clone());
            plan.push(Planned { version, dependency: true });
        }
    }
    Ok((plan, missing))
}

fn checksum_of(file: &VersionFile) -> Option<Checksum> {
    if !file.hashes.sha512.is_empty() {
        Some(Checksum::Sha512(file.hashes.sha512.clone()))
    } else if !file.hashes.sha1.is_empty() {
        Some(Checksum::Sha1(file.hashes.sha1.clone()))
    } else {
        None
    }
}

fn entry_for(planned: &Planned, file: &VersionFile, project: Option<&Project>, kind: ContentKind) -> ContentEntry {
    ContentEntry {
        kind,
        file_name: file.filename.clone(),
        title: project.map_or_else(|| file.filename.clone(), |project| project.title.clone()),
        project_id: Some(planned.version.project_id.clone()),
        version_id: Some(planned.version.id.clone()),
        version_number: Some(planned.version.version_number.clone()),
        icon_url: project.and_then(|project| project.icon_url.clone()),
        sha512: Some(file.hashes.sha512.clone()).filter(|hash| !hash.is_empty()),
        download_url: Some(file.url.clone()),
        size: Some(file.size),
        client_side: project.and_then(|project| project.client_side.clone()),
        server_side: project.and_then(|project| project.server_side.clone()),
        dependency: planned.dependency,
        installed_at: Utc::now(),
    }
}

/// Installs a project into an instance together with the dependencies it requires.
/// With `pinned_version` set that exact version is installed (used for updates).
pub async fn install_project(
    ctx: &Context<'_>,
    instance: &Instance,
    project_id: &str,
    pinned_version: Option<&str>,
    cancel: &CancellationToken,
    mut on_progress: impl FnMut(u64, u64),
) -> AppResult<InstallReport> {
    let project = ctx.api.project(project_id).await?;
    let kind = ContentKind::from_project_type(&project.project_type)
        .ok_or_else(|| AppError::InvalidInput("modpacks are installed as new instances".into()))?;
    let filter = version_filter(instance, kind)?;

    let root = match pinned_version {
        Some(version_id) => {
            let version = ctx.api.version(version_id).await?;
            if version.project_id != project.id {
                return Err(AppError::InvalidInput("the version belongs to a different project".into()));
            }
            version
        }
        None => pick_version(ctx.api.versions(&project.id, &filter).await?)
            .ok_or_else(|| AppError::NoCompatibleVersion(project.title.clone()))?,
    };

    let (paths, instance_id) = (ctx.paths.clone(), instance.id.clone());
    let existing = blocking(move || content::load_entries(&paths, &instance_id)).await?;
    let installed_mods: HashSet<String> = existing
        .iter()
        .filter(|entry| entry.kind == ContentKind::Mod)
        .filter_map(|entry| entry.project_id.clone())
        .collect();
    let (plan, missing_dependencies) = plan_installs(ctx, root, kind, &filter, &installed_mods).await?;

    let project_ids: Vec<String> = plan.iter().map(|planned| planned.version.project_id.clone()).collect();
    let projects: HashMap<String, Project> = ctx
        .api
        .projects(&project_ids)
        .await?
        .into_iter()
        .map(|project| (project.id.clone(), project))
        .collect();

    let game_dir = ctx.paths.instance_game_dir(&instance.id);
    let mut items = Vec::new();
    let mut entries = Vec::new();
    for planned in &plan {
        let file = planned.version.primary_file().ok_or_else(|| AppError::NoCompatibleVersion(planned.version.name.clone()))?;
        if !is_safe_file_name(&file.filename) {
            return Err(AppError::InvalidInput(format!("unsafe file name {}", file.filename)));
        }
        if !is_allowed_download_url(&file.url) {
            return Err(AppError::InvalidInput(format!("downloads from {} are not allowed", file.url)));
        }
        let info = projects.get(&planned.version.project_id);
        let entry_kind = info
            .and_then(|project| ContentKind::from_project_type(&project.project_type))
            .unwrap_or(kind);
        items.push(DownloadItem {
            url: file.url.clone(),
            path: game_dir.join(entry_kind.folder()).join(&file.filename),
            checksum: checksum_of(file),
            size: Some(file.size).filter(|size| *size > 0),
        });
        entries.push(entry_for(planned, file, info, entry_kind));
    }

    let total: u64 = items.iter().filter_map(|item| item.size).sum();
    download_with_progress(ctx.api.download_client(), items, cancel, |counters| on_progress(counters.bytes(), total)).await?;

    let (paths, instance_id, recorded) = (ctx.paths.clone(), instance.id.clone(), entries.clone());
    blocking(move || record_installed(&paths, &instance_id, recorded)).await?;
    Ok(InstallReport { installed: entries, missing_dependencies })
}

/// Replaces older entries of the same project (deleting their files) with the new ones.
fn record_installed(paths: &DataPaths, instance_id: &str, new_entries: Vec<ContentEntry>) -> AppResult<()> {
    let mut entries = content::load_entries(paths, instance_id)?;
    for new in &new_entries {
        let (stale, kept): (Vec<_>, Vec<_>) = entries.into_iter().partition(|entry| {
            entry.kind == new.kind
                && (entry.file_name == new.file_name || (new.project_id.is_some() && entry.project_id == new.project_id))
        });
        entries = kept;
        for old in stale.iter().filter(|old| old.file_name != new.file_name) {
            content::remove_files(&paths.instance_game_dir(instance_id).join(old.kind.folder()), &old.file_name);
        }
    }
    entries.extend(new_entries);
    content::save_entries(paths, instance_id, entries)
}

pub async fn check_updates(ctx: &Context<'_>, instance: &Instance) -> AppResult<Vec<UpdateInfo>> {
    let (paths, instance_id) = (ctx.paths.clone(), instance.id.clone());
    let entries = blocking(move || content::load_entries(&paths, &instance_id)).await?;

    let mut updates = Vec::new();
    for kind in ContentKind::ALL {
        let Ok(filter) = version_filter(instance, kind) else { continue };
        let tracked: Vec<&ContentEntry> = entries
            .iter()
            .filter(|entry| entry.kind == kind && entry.project_id.is_some() && entry.sha512.is_some())
            .collect();
        let hashes: Vec<String> = tracked.iter().filter_map(|entry| entry.sha512.clone()).collect();
        let latest = ctx.api.latest_from_hashes(&hashes, "sha512", &filter).await?;

        for entry in tracked {
            let Some(newest) = entry.sha512.as_ref().and_then(|hash| latest.get(hash)) else { continue };
            if Some(&newest.id) != entry.version_id.as_ref() {
                updates.push(UpdateInfo {
                    kind,
                    file_name: entry.file_name.clone(),
                    title: entry.title.clone(),
                    project_id: newest.project_id.clone(),
                    current_version: entry.version_number.clone(),
                    new_version: newest.version_number.clone(),
                    new_version_id: newest.id.clone(),
                });
            }
        }
    }
    Ok(updates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::model::{IconInput, NewInstance};
    use crate::instances::store;
    use crate::test_support::{start, Mock};
    use sha2::{Digest, Sha512};
    use std::fs;

    fn sha512(text: &str) -> String {
        hex::encode(Sha512::digest(text.as_bytes()))
    }

    fn version_json(id: &str, project: &str, file: &str, content: &str, dependencies: &str) -> String {
        format!(
            r#"{{"id":"{id}","project_id":"{project}","name":"{id}","version_number":"1.0-{id}","version_type":"release",
            "game_versions":["1.21.1"],"loaders":["fabric"],"dependencies":{dependencies},
            "files":[{{"hashes":{{"sha1":"","sha512":"{}"}},"url":"{{base}}/files/{file}","filename":"{file}","primary":true,"size":{}}}]}}"#,
            sha512(content),
            content.len()
        )
    }

    fn project_json(id: &str, title: &str) -> String {
        format!(r#"{{"id":"{id}","slug":"{id}","title":"{title}","description":"d","project_type":"mod","downloads":1,"followers":1,"client_side":"required","server_side":"optional"}}"#)
    }

    struct Fixture {
        mock: Mock,
        api: ModrinthApi,
        paths: DataPaths,
        instance: Instance,
    }

    async fn fixture(name: &str, routes: Vec<(&'static str, Vec<(u16, String)>)>) -> Fixture {
        let mock = start(routes).await;
        let api = ModrinthApi::with_base(reqwest::Client::new(), &mock.base, None);
        let paths = DataPaths::new(std::env::temp_dir().join(format!("zenith-modinstall-{name}-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        let new = NewInstance {
            name: "Modded".into(),
            game_version: "1.21.1".into(),
            icon: IconInput::Preset { key: "cube".into() },
            loader: LoaderKind::Fabric,
            loader_version: Some("0.16.14".into()),
        };
        let instance = store::create(&paths, new).unwrap();
        Fixture { mock, api, paths, instance }
    }

    fn ok(body: String) -> Vec<(u16, String)> {
        vec![(200, body)]
    }

    fn base_routes() -> Vec<(&'static str, Vec<(u16, String)>)> {
        let required = r#"[{"project_id":"LIB","version_id":null,"dependency_type":"required"},
            {"project_id":"EXTRA","version_id":null,"dependency_type":"optional"}]"#;
        vec![
            ("/project/MOD", ok(project_json("MOD", "Cool Mod"))),
            ("/project/MOD/version", ok(format!("[{}]", version_json("v-mod", "MOD", "cool.jar", "cool-bytes", required)))),
            ("/project/LIB/version", ok(format!("[{}]", version_json("v-lib", "LIB", "lib.jar", "lib-bytes", "[]")))),
            ("/projects", ok(format!("[{},{}]", project_json("MOD", "Cool Mod"), project_json("LIB", "Cool Lib")))),
            ("/files/cool.jar", ok("cool-bytes".into())),
            ("/files/lib.jar", ok("lib-bytes".into())),
        ]
    }

    async fn install(fixture: &Fixture, project: &str, pinned: Option<&str>) -> AppResult<InstallReport> {
        let context = Context { api: &fixture.api, paths: &fixture.paths };
        install_project(&context, &fixture.instance, project, pinned, &CancellationToken::new(), |_, _| {}).await
    }

    #[tokio::test]
    async fn installs_a_mod_with_its_required_dependencies_only() {
        let fixture = fixture("deps", base_routes()).await;

        let report = install(&fixture, "MOD", None).await.unwrap();

        let mods = fixture.paths.instance_game_dir(&fixture.instance.id).join("mods");
        assert_eq!(fs::read_to_string(mods.join("cool.jar")).unwrap(), "cool-bytes");
        assert_eq!(fs::read_to_string(mods.join("lib.jar")).unwrap(), "lib-bytes");
        assert_eq!(report.installed.len(), 2);
        let entries = content::load_entries(&fixture.paths, &fixture.instance.id).unwrap();
        let lib = entries.iter().find(|entry| entry.project_id.as_deref() == Some("LIB")).unwrap();
        assert!(lib.dependency && lib.title == "Cool Lib");
        assert!(!entries.iter().find(|entry| entry.file_name == "cool.jar").unwrap().dependency);
        assert_eq!(fixture.mock.count("/project/EXTRA/version"), 0);
        assert!(fixture.mock.last("/project/LIB/version").query.contains("loaders=%5B%22fabric%22%5D"));
        let _ = fs::remove_dir_all(fixture.paths.root());
    }

    #[tokio::test]
    async fn skips_dependencies_that_are_already_installed() {
        let fixture = fixture("skip-dep", base_routes()).await;
        let existing = ContentEntry {
            kind: ContentKind::Mod,
            file_name: "lib.jar".into(),
            title: "Cool Lib".into(),
            project_id: Some("LIB".into()),
            version_id: Some("v-lib".into()),
            version_number: None,
            icon_url: None,
            sha512: None,
            download_url: None,
            size: None,
            client_side: None,
            server_side: None,
            dependency: true,
            installed_at: Utc::now(),
        };
        content::save_entries(&fixture.paths, &fixture.instance.id, vec![existing]).unwrap();

        let report = install(&fixture, "MOD", None).await.unwrap();

        assert_eq!(report.installed.len(), 1);
        assert_eq!(fixture.mock.count("/project/LIB/version"), 0);
        let _ = fs::remove_dir_all(fixture.paths.root());
    }

    #[tokio::test]
    async fn fails_clearly_when_no_version_fits_the_instance() {
        let mut routes = base_routes();
        routes[1] = ("/project/MOD/version", ok("[]".into()));
        let fixture = fixture("no-version", routes).await;

        let result = install(&fixture, "MOD", None).await;

        assert!(matches!(result, Err(AppError::NoCompatibleVersion(title)) if title == "Cool Mod"));
        let _ = fs::remove_dir_all(fixture.paths.root());
    }

    #[tokio::test]
    async fn rejects_files_that_do_not_match_their_published_hash() {
        let mut routes = base_routes();
        routes[4] = ("/files/cool.jar", ok("tampered".into()));
        let fixture = fixture("tampered", routes).await;

        let result = install(&fixture, "MOD", None).await;

        assert!(result.is_err());
        assert!(content::load_entries(&fixture.paths, &fixture.instance.id).unwrap().is_empty());
        let _ = fs::remove_dir_all(fixture.paths.root());
    }

    #[tokio::test]
    async fn refuses_downloads_from_hosts_outside_the_allow_list_and_unsafe_names() {
        assert!(is_allowed_download_url("https://cdn.modrinth.com/data/x/versions/y/a.jar"));
        assert!(is_allowed_download_url("https://raw.githubusercontent.com/a/b/c.jar"));
        assert!(!is_allowed_download_url("http://cdn.modrinth.com/a.jar"));
        assert!(!is_allowed_download_url("https://evil.example/a.jar"));
        assert!(!is_allowed_download_url("not a url"));

        let mut routes = base_routes();
        let evil = version_json("v-mod", "MOD", "cool.jar", "cool-bytes", "[]").replace("{base}", "https://evil.example");
        routes[1] = ("/project/MOD/version", ok(format!("[{evil}]")));
        let fixture = fixture("evil-host", routes).await;
        assert!(matches!(install(&fixture, "MOD", None).await, Err(AppError::InvalidInput(_))));

        let mut routes = base_routes();
        let traversal = version_json("v-mod", "MOD", "cool.jar", "cool-bytes", "[]").replace(r#""filename":"cool.jar""#, r#""filename":"../cool.jar""#);
        routes[1] = ("/project/MOD/version", ok(format!("[{traversal}]")));
        let fixture = self::fixture("evil-name", routes).await;
        assert!(matches!(install(&fixture, "MOD", None).await, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn updating_replaces_the_old_file_and_keeps_a_single_entry() {
        let mut routes = base_routes();
        routes[1] = ("/project/MOD/version", ok(format!("[{}]", version_json("v-mod", "MOD", "cool.jar", "cool-bytes", "[]"))));
        routes.push(("/version/v-new", ok(version_json("v-new", "MOD", "cool-2.jar", "cool-2-bytes", "[]"))));
        routes.push(("/files/cool-2.jar", ok("cool-2-bytes".into())));
        let fixture = fixture("update", routes).await;
        install(&fixture, "MOD", None).await.unwrap();

        install(&fixture, "MOD", Some("v-new")).await.unwrap();

        let mods = fixture.paths.instance_game_dir(&fixture.instance.id).join("mods");
        assert!(!mods.join("cool.jar").exists());
        assert!(mods.join("cool-2.jar").exists());
        let entries = content::load_entries(&fixture.paths, &fixture.instance.id).unwrap();
        assert_eq!(entries.iter().filter(|entry| entry.project_id.as_deref() == Some("MOD")).count(), 1);
        assert_eq!(entries[0].version_id.as_deref(), Some("v-new"));
        let _ = fs::remove_dir_all(fixture.paths.root());
    }

    #[tokio::test]
    async fn finds_available_updates_by_file_hash() {
        let hash = sha512("cool-bytes");
        let newer = version_json("v-newer", "MOD", "cool-3.jar", "cool-3-bytes", "[]");
        let update = format!(r#"{{"{hash}":{newer}}}"#);
        let mut routes = base_routes();
        routes.push(("/version_files/update", ok(update)));
        let fixture = fixture("check", routes).await;
        install(&fixture, "MOD", None).await.unwrap();

        let context = Context { api: &fixture.api, paths: &fixture.paths };
        let updates = check_updates(&context, &fixture.instance).await.unwrap();

        assert_eq!(updates.len(), 1);
        assert_eq!((updates[0].file_name.as_str(), updates[0].new_version_id.as_str()), ("cool.jar", "v-newer"));
        let body = fixture.mock.last("/version_files/update").body;
        assert!(body.contains(&hash) && body.contains("\"fabric\"") && body.contains("1.21.1"));
        let _ = fs::remove_dir_all(fixture.paths.root());
    }

    #[tokio::test]
    async fn mods_need_a_mod_loader_but_resource_packs_do_not() {
        let mut vanilla = fixture("vanilla", base_routes()).await.instance;
        vanilla.loader = LoaderKind::Vanilla;

        assert!(version_filter(&vanilla, ContentKind::Mod).is_err());
        assert_eq!(version_filter(&vanilla, ContentKind::ResourcePack).unwrap().loaders, ["minecraft"]);
        vanilla.loader = LoaderKind::Quilt;
        assert_eq!(version_filter(&vanilla, ContentKind::Mod).unwrap().loaders, ["quilt", "fabric"]);
    }

    #[tokio::test]
    #[ignore = "talks to the real Modrinth API and downloads mods"]
    async fn installs_real_mods_with_dependencies_and_finds_no_updates() {
        let api = ModrinthApi::new(reqwest::Client::new(), None);
        let paths = DataPaths::new(std::env::temp_dir().join(format!("zenith-live-mods-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        let new = NewInstance {
            name: "Live".into(),
            game_version: "1.21.1".into(),
            icon: IconInput::Preset { key: "cube".into() },
            loader: LoaderKind::Fabric,
            loader_version: Some("0.16.14".into()),
        };
        let instance = store::create(&paths, new).unwrap();
        let context = Context { api: &api, paths: &paths };

        let found = api
            .search(&crate::modrinth::api::SearchQuery {
                query: "mod menu".into(),
                project_type: Some(crate::modrinth::types::ProjectType::Mod),
                game_version: Some("1.21.1".into()),
                loader: Some("fabric".into()),
                ..Default::default()
            })
            .await
            .unwrap();
        println!("search: {} hits, first: {}", found.total_hits, found.hits[0].title);
        let modmenu = found.hits.iter().find(|hit| hit.slug == "modmenu").expect("Mod Menu should be found");

        let report = install_project(&context, &instance, &modmenu.project_id, None, &CancellationToken::new(), |_, _| {})
            .await
            .unwrap();

        for entry in &report.installed {
            println!("installed {} ({}) dependency={}", entry.title, entry.file_name, entry.dependency);
        }
        assert!(report.installed.iter().any(|entry| entry.dependency), "Mod Menu requires Fabric API");
        let mods = paths.instance_game_dir(&instance.id).join("mods");
        assert!(report.installed.iter().all(|entry| mods.join(&entry.file_name).is_file()));
        let updates = check_updates(&context, &instance).await.unwrap();
        println!("updates available right after install: {}", updates.len());
        assert!(updates.is_empty());
        let _ = fs::remove_dir_all(paths.root());
    }}
