use crate::error::{AppError, AppResult};
use crate::instances::model::{Instance, LoaderKind};
use crate::modrinth::content::{ContentEntry, ContentKind};
use crate::mojang::profile::is_safe_id;
use base64::Engine;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const PREFIX: &str = "KAIROS1-";
const MAX_CODE_CHARS: usize = 64 * 1024;
const MAX_JSON_BYTES: u64 = 256 * 1024;
const MAX_ITEMS: usize = 300;
const MAX_NAME_CHARS: usize = 64;
const MAX_ID_CHARS: usize = 24;

/// One piece of content: its kind, the project and version it came from, and whether it was only pulled in as a dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item(pub u8, pub String, pub String, pub bool);

impl Item {
    pub fn kind(&self) -> Option<ContentKind> {
        match self.0 {
            0 => Some(ContentKind::Mod),
            1 => Some(ContentKind::ResourcePack),
            2 => Some(ContentKind::Shader),
            _ => None,
        }
    }

    pub fn project(&self) -> &str {
        &self.1
    }

    pub fn version(&self) -> &str {
        &self.2
    }

    pub fn is_dependency(&self) -> bool {
        self.3
    }
}

/// What a code carries: enough to rebuild an instance with the same game version, loader and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub n: String,
    pub g: String,
    pub l: LoaderKind,
    pub lv: Option<String>,
    pub i: Vec<Item>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePreview {
    pub name: String,
    pub game_version: String,
    pub loader: LoaderKind,
    pub loader_version: Option<String>,
    pub mods: u32,
    pub resource_packs: u32,
    pub shaders: u32,
}

fn invalid(message: &str) -> AppError {
    AppError::InvalidInput(message.to_owned())
}

fn kind_code(kind: ContentKind) -> u8 {
    match kind {
        ContentKind::Mod => 0,
        ContentKind::ResourcePack => 1,
        ContentKind::Shader => 2,
    }
}

fn is_catalogue_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_ID_CHARS && id.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

/// Turns an instance and the content Kairos installed into it into a short text code.
pub fn encode(instance: &Instance, entries: &[ContentEntry]) -> AppResult<String> {
    let items: Vec<Item> = entries
        .iter()
        .filter_map(|entry| Some(Item(kind_code(entry.kind), entry.project_id.clone()?, entry.version_id.clone()?, entry.dependency)))
        .collect();
    let payload = Payload {
        n: instance.name.chars().take(MAX_NAME_CHARS).collect(),
        g: instance.game_version.clone(),
        l: instance.loader,
        lv: instance.loader_version.clone(),
        i: items,
    };
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&serde_json::to_vec(&payload)?)?;
    let packed = encoder.finish()?;
    Ok(format!("{PREFIX}{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(packed)))
}

/// Reads a code and checks everything in it, since it can come from anyone.
pub fn decode(code: &str) -> AppResult<Payload> {
    let code: String = code.chars().filter(|character| !character.is_whitespace()).collect();
    if code.len() > MAX_CODE_CHARS {
        return Err(invalid("this code is too long"));
    }
    let body = code.strip_prefix(PREFIX).ok_or_else(|| invalid("this is not a Kairos code"))?;
    let packed = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(body).map_err(|_| invalid("this code is damaged"))?;
    let mut json = Vec::new();
    DeflateDecoder::new(packed.as_slice()).take(MAX_JSON_BYTES + 1).read_to_end(&mut json).map_err(|_| invalid("this code is damaged"))?;
    if json.len() as u64 > MAX_JSON_BYTES {
        return Err(invalid("this code is too large"));
    }
    let payload: Payload = serde_json::from_slice(&json).map_err(|_| invalid("this code is damaged"))?;
    validate(&payload)?;
    Ok(payload)
}

fn validate(payload: &Payload) -> AppResult<()> {
    let name_ok = !payload.n.trim().is_empty() && payload.n.chars().count() <= MAX_NAME_CHARS && !payload.n.chars().any(char::is_control);
    let loader_ok = match (payload.l, &payload.lv) {
        (LoaderKind::Vanilla, None) => true,
        (LoaderKind::Vanilla, Some(_)) | (_, None) => false,
        (_, Some(version)) => is_safe_id(version),
    };
    let items_ok = payload.i.len() <= MAX_ITEMS && payload.i.iter().all(|item| item.kind().is_some() && is_catalogue_id(item.project()) && is_catalogue_id(item.version()));
    if name_ok && is_safe_id(&payload.g) && loader_ok && items_ok {
        Ok(())
    } else {
        Err(invalid("this code is not valid"))
    }
}

pub fn preview(payload: &Payload) -> CodePreview {
    let count = |kind: ContentKind| u32::try_from(payload.i.iter().filter(|item| item.kind() == Some(kind)).count()).unwrap_or(u32::MAX);
    CodePreview {
        name: payload.n.clone(),
        game_version: payload.g.clone(),
        loader: payload.l,
        loader_version: payload.lv.clone(),
        mods: count(ContentKind::Mod),
        resource_packs: count(ContentKind::ResourcePack),
        shaders: count(ContentKind::Shader),
    }
}

/// Dependencies go first, so that the mods that need them find them already installed.
pub fn install_order(payload: &Payload) -> Vec<&Item> {
    let mut items: Vec<&Item> = payload.i.iter().collect();
    items.sort_by_key(|item| !item.is_dependency());
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::model::InstanceIcon;
    use chrono::Utc;

    fn instance(loader: LoaderKind, loader_version: Option<&str>) -> Instance {
        Instance {
            schema_version: 1,
            id: "demo".into(),
            name: "My pack".into(),
            icon: InstanceIcon::Preset { key: "cube".into() },
            game_version: "1.21.1".into(),
            loader,
            loader_version: loader_version.map(str::to_owned),
            memory_mb: 4096,
            jvm_args: Vec::new(),
            java_path: None,
            created_at: Utc::now(),
            last_played_at: None,
            play_time_secs: 0,
        }
    }

    fn entry(kind: ContentKind, project: &str, version: &str, dependency: bool) -> ContentEntry {
        ContentEntry {
            kind,
            file_name: format!("{project}.jar"),
            title: project.into(),
            project_id: Some(project.into()),
            version_id: Some(version.into()),
            version_number: None,
            icon_url: None,
            sha512: None,
            download_url: None,
            size: None,
            client_side: None,
            server_side: None,
            dependency,
            installed_at: Utc::now(),
        }
    }

    fn code_for(payload: &Payload) -> String {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&serde_json::to_vec(payload).unwrap()).unwrap();
        format!("{PREFIX}{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(encoder.finish().unwrap()))
    }

    #[test]
    fn a_code_carries_the_instance_and_its_content_and_stays_short() {
        let entries = vec![
            entry(ContentKind::Mod, "AANobbMI", "u1pctuWU", false),
            entry(ContentKind::Mod, "P7dR8mSH", "Qq8cbGRu", true),
            entry(ContentKind::Shader, "YL57xq9U", "abc12345", false),
            ContentEntry { project_id: None, ..entry(ContentKind::Mod, "hand", "x", false) },
        ];
        let code = encode(&instance(LoaderKind::Fabric, Some("0.16.14")), &entries).unwrap();
        assert!(code.starts_with(PREFIX) && code.len() < 400, "{}", code.len());

        let payload = decode(&format!("  {}\n", &code)).unwrap();
        assert_eq!((payload.n.as_str(), payload.g.as_str(), payload.l, payload.lv.as_deref()), ("My pack", "1.21.1", LoaderKind::Fabric, Some("0.16.14")));
        assert_eq!(payload.i.len(), 3, "files that were put in by hand cannot be shared");
        let shown = preview(&payload);
        assert_eq!((shown.mods, shown.resource_packs, shown.shaders), (2, 0, 1));
        assert!(install_order(&payload)[0].is_dependency());
    }

    #[test]
    fn many_mods_still_fit_in_a_chat_message() {
        let entries: Vec<ContentEntry> = (0..60).map(|index| entry(ContentKind::Mod, &format!("Proj{index:04}"), &format!("Vers{index:04}"), index % 3 == 0)).collect();
        let code = encode(&instance(LoaderKind::Fabric, Some("0.16.14")), &entries).unwrap();
        assert!(code.len() < 2000, "{}", code.len());
    }

    #[test]
    fn refuses_codes_that_are_wrong_damaged_or_hostile() {
        assert!(decode("hello").is_err());
        assert!(decode("KAIROS1-!!!").is_err());
        assert!(decode(&format!("{PREFIX}{}", "A".repeat(200))).is_err());

        let good = Payload { n: "ok".into(), g: "1.21.1".into(), l: LoaderKind::Vanilla, lv: None, i: Vec::new() };
        assert!(decode(&code_for(&good)).is_ok());
        for bad in [
            Payload { g: "../etc".into(), ..good.clone() },
            Payload { n: "  ".into(), ..good.clone() },
            Payload { l: LoaderKind::Fabric, lv: None, ..good.clone() },
            Payload { lv: Some("1".into()), ..good.clone() },
            Payload { i: vec![Item(9, "AANobbMI".into(), "abc".into(), false)], ..good.clone() },
            Payload { i: vec![Item(0, "bad id!".into(), "abc".into(), false)], ..good.clone() },
            Payload { i: (0..301).map(|index| Item(0, format!("P{index}"), "V".into(), false)).collect(), ..good.clone() },
        ] {
            assert!(decode(&code_for(&bad)).is_err());
        }

        let mut bomb = DeflateEncoder::new(Vec::new(), Compression::best());
        bomb.write_all(&vec![b' '; 2 * 1024 * 1024]).unwrap();
        let bomb = format!("{PREFIX}{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bomb.finish().unwrap()));
        assert!(bomb.len() < MAX_CODE_CHARS && decode(&bomb).is_err(), "an archive bomb must not be unpacked");
    }
}
