use serde::Deserialize;
use std::collections::HashMap;

pub const ASSET_BASE_URL: &str = "https://resources.download.minecraft.net";

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
    #[serde(default, rename = "virtual")]
    pub is_virtual: bool,
    #[serde(default)]
    pub map_to_resources: bool,
}

#[derive(Debug, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

impl AssetObject {
    pub fn url(&self) -> String {
        format!("{ASSET_BASE_URL}/{}/{}", &self.hash[..2], self.hash)
    }
}
