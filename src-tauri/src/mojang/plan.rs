use super::assets::AssetIndex;
use super::version::VersionJson;
use crate::download::DownloadItem;
use crate::paths::DataPaths;
use std::collections::HashSet;

pub fn core_items(version: &VersionJson, paths: &DataPaths) -> Vec<DownloadItem> {
    let mut items = vec![DownloadItem {
        url: version.downloads.client.url.clone(),
        path: paths.client_jar(&version.id),
        sha1: Some(version.downloads.client.sha1.clone()),
        size: Some(version.downloads.client.size),
    }];

    for library in version.libraries.iter().filter(|library| library.applies()) {
        let Some(downloads) = &library.downloads else {
            continue;
        };
        let classifier = library
            .native_classifier()
            .and_then(|key| downloads.classifiers.as_ref()?.get(&key));

        for artifact in downloads.artifact.iter().chain(classifier) {
            items.push(DownloadItem {
                url: artifact.url.clone(),
                path: paths.libraries().join(&artifact.path),
                sha1: Some(artifact.sha1.clone()),
                size: Some(artifact.size),
            });
        }
    }

    if let Some(file) = version.logging.as_ref().and_then(|l| l.client.as_ref()).map(|c| &c.file) {
        items.push(DownloadItem {
            url: file.url.clone(),
            path: paths.log_configs().join(&file.id),
            sha1: Some(file.sha1.clone()),
            size: Some(file.size),
        });
    }

    items
}

fn is_sha1_hex(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn asset_items(index: &AssetIndex, paths: &DataPaths) -> Vec<DownloadItem> {
    let mut seen = HashSet::new();
    index
        .objects
        .values()
        .filter(|object| is_sha1_hex(&object.hash) && seen.insert(object.hash.as_str()))
        .map(|object| DownloadItem {
            url: object.url(),
            path: paths.asset_object(&object.hash),
            sha1: Some(object.hash.clone()),
            size: Some(object.size),
        })
        .collect()
}
