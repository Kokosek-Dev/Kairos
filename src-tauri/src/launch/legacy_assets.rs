use crate::error::AppResult;
use crate::mojang::assets::AssetIndex;
use crate::paths::DataPaths;
use std::path::{Component, Path, PathBuf};

/// Pre-1.7 versions read assets by file name instead of by hash. Returns the directory
/// that must be passed to the game as its assets directory.
pub fn prepare(index: &AssetIndex, index_id: &str, paths: &DataPaths, game_dir: &Path) -> AppResult<PathBuf> {
    let target = if index.map_to_resources {
        game_dir.join("resources")
    } else if index.is_virtual {
        paths.virtual_assets(index_id)
    } else {
        return Ok(paths.assets());
    };

    for (name, object) in &index.objects {
        let relative = Path::new(name);
        if !relative.components().all(|part| matches!(part, Component::Normal(_))) || object.hash.len() < 2 {
            continue;
        }
        let destination = target.join(relative);
        let source = paths.asset_object(&object.hash);
        if destination.metadata().is_ok_and(|meta| meta.len() == object.size) {
            continue;
        }
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source, destination)?;
    }
    Ok(target)
}
