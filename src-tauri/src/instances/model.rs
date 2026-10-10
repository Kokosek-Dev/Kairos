use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_MEMORY_MB: u32 = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderKind {
    Vanilla,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstanceIcon {
    Preset { key: String },
    Custom { file: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub icon: InstanceIcon,
    pub game_version: String,
    pub loader: LoaderKind,
    pub memory_mb: u32,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    pub java_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_played_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IconInput {
    Preset { key: String },
    Custom { path: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInstance {
    pub name: String,
    pub game_version: String,
    pub icon: IconInput,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSettings {
    pub name: String,
    pub game_version: String,
    pub memory_mb: u32,
    pub jvm_args: Vec<String>,
    pub java_path: Option<String>,
}
