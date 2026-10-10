use serde::{Deserialize, Serialize};

/// Modrinth's JSON uses snake_case; the frontend receives the same data as camelCase.
macro_rules! api_type {
    ($(#[$meta:meta])* pub struct $name:ident { $($body:tt)* }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all(deserialize = "snake_case", serialize = "camelCase"))]
        pub struct $name { $($body)* }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    Mod,
    Modpack,
    Resourcepack,
    Shader,
}

impl ProjectType {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Mod => "mod",
            Self::Modpack => "modpack",
            Self::Resourcepack => "resourcepack",
            Self::Shader => "shader",
        }
    }
}

api_type! {
    pub struct SearchHit {
        pub project_id: String,
        pub slug: String,
        pub project_type: String,
        pub title: String,
        pub description: String,
        pub author: String,
        #[serde(default)]
        pub icon_url: Option<String>,
        pub downloads: u64,
        pub follows: u64,
        #[serde(default)]
        pub display_categories: Vec<String>,
        #[serde(default)]
        pub client_side: Option<String>,
        #[serde(default)]
        pub server_side: Option<String>,
        #[serde(default)]
        pub date_modified: String,
    }
}

api_type! {
    pub struct SearchResults {
        pub hits: Vec<SearchHit>,
        pub offset: u32,
        pub limit: u32,
        pub total_hits: u32,
    }
}

api_type! {
    pub struct GalleryImage {
        pub url: String,
        #[serde(default)]
        pub title: Option<String>,
        #[serde(default)]
        pub featured: bool,
    }
}

api_type! {
    pub struct License {
        #[serde(default)]
        pub id: String,
        #[serde(default)]
        pub name: String,
    }
}

api_type! {
    pub struct Project {
        pub id: String,
        pub slug: String,
        pub title: String,
        pub description: String,
        #[serde(default)]
        pub body: String,
        pub project_type: String,
        #[serde(default)]
        pub icon_url: Option<String>,
        #[serde(default)]
        pub categories: Vec<String>,
        #[serde(default)]
        pub loaders: Vec<String>,
        #[serde(default)]
        pub game_versions: Vec<String>,
        pub downloads: u64,
        pub followers: u64,
        #[serde(default)]
        pub client_side: Option<String>,
        #[serde(default)]
        pub server_side: Option<String>,
        #[serde(default)]
        pub license: Option<License>,
        #[serde(default)]
        pub issues_url: Option<String>,
        #[serde(default)]
        pub source_url: Option<String>,
        #[serde(default)]
        pub wiki_url: Option<String>,
        #[serde(default)]
        pub discord_url: Option<String>,
        #[serde(default)]
        pub gallery: Vec<GalleryImage>,
    }
}

api_type! {
    pub struct Hashes {
        #[serde(default)]
        pub sha1: String,
        #[serde(default)]
        pub sha512: String,
    }
}

api_type! {
    pub struct VersionFile {
        pub hashes: Hashes,
        pub url: String,
        pub filename: String,
        #[serde(default)]
        pub primary: bool,
        #[serde(default)]
        pub size: u64,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

api_type! {
    pub struct Dependency {
        #[serde(default)]
        pub version_id: Option<String>,
        #[serde(default)]
        pub project_id: Option<String>,
        pub dependency_type: DependencyType,
    }
}

api_type! {
    pub struct Version {
        pub id: String,
        pub project_id: String,
        pub name: String,
        pub version_number: String,
        #[serde(default)]
        pub changelog: Option<String>,
        pub version_type: String,
        #[serde(default)]
        pub game_versions: Vec<String>,
        #[serde(default)]
        pub loaders: Vec<String>,
        #[serde(default)]
        pub date_published: String,
        #[serde(default)]
        pub downloads: u64,
        #[serde(default)]
        pub files: Vec<VersionFile>,
        #[serde(default)]
        pub dependencies: Vec<Dependency>,
    }
}

impl Version {
    pub fn primary_file(&self) -> Option<&VersionFile> {
        self.files.iter().find(|file| file.primary).or_else(|| self.files.first())
    }
}

api_type! {
    pub struct TeamUser {
        pub username: String,
        #[serde(default)]
        pub avatar_url: Option<String>,
    }
}

api_type! {
    pub struct TeamMember {
        pub user: TeamUser,
        #[serde(default)]
        pub role: String,
    }
}

api_type! {
    pub struct Category {
        pub name: String,
        pub project_type: String,
        #[serde(default)]
        pub header: String,
    }
}
