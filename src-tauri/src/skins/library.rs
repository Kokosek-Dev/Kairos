use crate::error::{AppError, AppResult};
use crate::paths::DataPaths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const MAX_SKIN_BYTES: usize = 128 * 1024;
const MAX_NAME_CHARS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkinVariant {
    Classic,
    Slim,
}

impl SkinVariant {
    pub fn api_name(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Slim => "slim",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSkin {
    pub id: String,
    pub name: String,
    pub variant: SkinVariant,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Index {
    skins: Vec<SavedSkin>,
}

fn invalid(message: &str) -> AppError {
    AppError::InvalidInput(message.to_owned())
}

/// Minecraft accepts 64x64 skins and the legacy 64x32 layout.
pub fn validate_png(bytes: &[u8]) -> AppResult<()> {
    if bytes.len() > MAX_SKIN_BYTES {
        return Err(invalid("the skin file is too large"));
    }
    if bytes.len() < 24 || bytes[..8] != PNG_SIGNATURE {
        return Err(invalid("the skin must be a PNG image"));
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width != 64 || !matches!(height, 32 | 64) {
        return Err(invalid("the skin must be 64x64 (or legacy 64x32) pixels"));
    }
    Ok(())
}

fn read_index(paths: &DataPaths) -> AppResult<Index> {
    match fs::read(paths.skins().join("skins.json")) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Index::default()),
        Err(error) => Err(error.into()),
    }
}

fn write_index(paths: &DataPaths, index: &Index) -> AppResult<()> {
    fs::create_dir_all(paths.skins())?;
    let target = paths.skins().join("skins.json");
    let temp = target.with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(index)?)?;
    fs::rename(temp, target)?;
    Ok(())
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn list(paths: &DataPaths) -> AppResult<Vec<SavedSkin>> {
    let mut skins = read_index(paths)?.skins;
    skins.sort_by(|a, b| b.added_at.cmp(&a.added_at));
    Ok(skins)
}

pub fn add(paths: &DataPaths, name: &str, variant: SkinVariant, png: &[u8]) -> AppResult<SavedSkin> {
    validate_png(png)?;
    let name: String = name.trim().chars().filter(|character| !character.is_control()).take(MAX_NAME_CHARS).collect();
    if name.is_empty() {
        return Err(invalid("give the skin a name"));
    }
    let mut id_bytes = [0u8; 8];
    getrandom::fill(&mut id_bytes).map_err(|error| AppError::Internal(error.to_string()))?;
    let skin = SavedSkin { id: hex::encode(id_bytes), name, variant, added_at: Utc::now() };

    fs::create_dir_all(paths.skins())?;
    fs::write(paths.skins().join(format!("{}.png", skin.id)), png)?;
    let mut index = read_index(paths)?;
    index.skins.push(skin.clone());
    write_index(paths, &index)?;
    Ok(skin)
}

pub fn get(paths: &DataPaths, id: &str) -> AppResult<(SavedSkin, Vec<u8>)> {
    if !is_valid_id(id) {
        return Err(AppError::SkinNotFound);
    }
    let skin = read_index(paths)?.skins.into_iter().find(|skin| skin.id == id).ok_or(AppError::SkinNotFound)?;
    let png = fs::read(paths.skins().join(format!("{id}.png")))?;
    Ok((skin, png))
}

pub fn remove(paths: &DataPaths, id: &str) -> AppResult<()> {
    if !is_valid_id(id) {
        return Err(AppError::SkinNotFound);
    }
    let mut index = read_index(paths)?;
    index.skins.retain(|skin| skin.id != id);
    write_index(paths, &index)?;
    let _ = fs::remove_file(paths.skins().join(format!("{id}.png")));
    Ok(())
}

#[cfg(test)]
pub fn test_png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = PNG_SIGNATURE.to_vec();
    bytes.extend_from_slice(&[0, 0, 0, 13, b'I', b'H', b'D', b'R']);
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> DataPaths {
        let paths = DataPaths::new(std::env::temp_dir().join(format!("kairos-skins-{name}-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        paths
    }

    #[test]
    fn accepts_only_real_minecraft_skin_dimensions() {
        assert!(validate_png(&test_png(64, 64)).is_ok());
        assert!(validate_png(&test_png(64, 32)).is_ok());
        assert!(validate_png(&test_png(128, 128)).is_err());
        assert!(validate_png(&test_png(64, 48)).is_err());
        assert!(validate_png(b"GIF89a not a png at all, definitely").is_err());
        assert!(validate_png(&[0u8; 200 * 1024]).is_err());
    }

    #[test]
    fn stores_lists_reads_and_removes_skins() {
        let paths = scratch("library");
        let png = test_png(64, 64);

        let first = add(&paths, "  Steve  ", SkinVariant::Classic, &png).unwrap();
        let second = add(&paths, "Alex", SkinVariant::Slim, &png).unwrap();

        assert_eq!(first.name, "Steve");
        assert_eq!(list(&paths).unwrap().len(), 2);
        let (loaded, bytes) = get(&paths, &second.id).unwrap();
        assert_eq!((loaded.variant, bytes), (SkinVariant::Slim, png));

        remove(&paths, &first.id).unwrap();
        assert_eq!(list(&paths).unwrap().len(), 1);
        assert!(matches!(get(&paths, &first.id), Err(AppError::SkinNotFound)));
        let _ = fs::remove_dir_all(paths.root());
    }

    #[test]
    fn rejects_bad_names_files_and_ids_that_could_escape_the_folder() {
        let paths = scratch("invalid");
        assert!(add(&paths, "   ", SkinVariant::Classic, &test_png(64, 64)).is_err());
        assert!(add(&paths, "x", SkinVariant::Classic, b"nope").is_err());
        for id in ["../x", "a/b", "", "zz", &"a".repeat(40)] {
            assert!(matches!(get(&paths, id), Err(AppError::SkinNotFound)), "{id}");
            assert!(matches!(remove(&paths, id), Err(AppError::SkinNotFound)), "{id}");
        }
    }
}
