use crate::error::{AppError, AppResult};
use crate::mojang::version::VersionJson;
use crate::paths::DataPaths;
use std::fs::{self, File};
use std::io;
use std::path::Path;

pub fn extract_natives(version: &VersionJson, paths: &DataPaths) -> AppResult<()> {
    let destination = paths.natives_dir(&version.id);
    if destination.exists() {
        fs::remove_dir_all(&destination)?;
    }
    fs::create_dir_all(&destination)?;

    for library in version.libraries.iter().filter(|library| library.applies()) {
        let Some(artifact) = library.native_artifact() else {
            continue;
        };
        let excluded = library.extract.as_ref().map(|extract| extract.exclude.as_slice()).unwrap_or_default();
        extract_jar(&paths.libraries().join(&artifact.path), &destination, excluded)?;
    }
    Ok(())
}

fn extract_jar(jar: &Path, destination: &Path, excluded: &[String]) -> AppResult<()> {
    let mut archive = zip::ZipArchive::new(File::open(jar)?).map_err(|error| AppError::Internal(error.to_string()))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| AppError::Internal(error.to_string()))?;
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        let name = relative.to_string_lossy().replace('\\', "/");
        if entry.is_dir() || excluded.iter().any(|prefix| name.starts_with(prefix.as_str())) {
            continue;
        }
        let target = destination.join(&relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        io::copy(&mut entry, &mut File::create(&target)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extracts_natives_skipping_excluded_prefixes_and_unsafe_paths() {
        let dir = std::env::temp_dir().join(format!("zenith-natives-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let jar = dir.join("natives.jar");

        let mut writer = zip::ZipWriter::new(File::create(&jar).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        for (name, body) in [("lwjgl.dll", "dll"), ("META-INF/MANIFEST.MF", "meta"), ("../evil.dll", "evil")] {
            writer.start_file(name, options).unwrap();
            writer.write_all(body.as_bytes()).unwrap();
        }
        writer.finish().unwrap();

        let out = dir.join("out");
        fs::create_dir_all(&out).unwrap();
        extract_jar(&jar, &out, &["META-INF/".to_owned()]).unwrap();

        assert_eq!(fs::read(out.join("lwjgl.dll")).unwrap(), b"dll");
        assert!(!out.join("META-INF").exists());
        assert!(!dir.join("evil.dll").exists());
        let _ = fs::remove_dir_all(dir);
    }
}
