use super::library::{validate_png, SkinVariant};
use crate::error::{AppError, AppResult};
use base64::Engine;
use reqwest::{multipart, Client, RequestBuilder, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const SERVICES: &str = "https://api.minecraftservices.com";
const MOJANG: &str = "https://api.mojang.com";
const SESSION: &str = "https://sessionserver.mojang.com";
const TEXTURE_HOST: &str = "textures.minecraft.net";
const MAX_TEXTURE_BYTES: usize = 256 * 1024;

#[derive(Debug, Deserialize)]
struct RawProfile {
    id: String,
    name: String,
    #[serde(default)]
    skins: Vec<RawSkin>,
    #[serde(default)]
    capes: Vec<RawCape>,
}

#[derive(Debug, Deserialize)]
struct RawSkin {
    state: String,
    url: String,
    variant: String,
}

#[derive(Debug, Deserialize)]
struct RawCape {
    id: String,
    state: String,
    url: String,
    #[serde(default)]
    alias: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapeView {
    pub id: String,
    pub alias: String,
    pub url: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView {
    pub id: String,
    pub name: String,
    pub skin_url: Option<String>,
    pub variant: SkinVariant,
    pub capes: Vec<CapeView>,
}

impl From<RawProfile> for ProfileView {
    fn from(raw: RawProfile) -> Self {
        let skin = raw.skins.iter().find(|skin| skin.state == "ACTIVE");
        Self {
            id: raw.id,
            name: raw.name,
            variant: if skin.is_some_and(|skin| skin.variant.eq_ignore_ascii_case("slim")) {
                SkinVariant::Slim
            } else {
                SkinVariant::Classic
            },
            skin_url: skin.map(|skin| https(&skin.url)),
            capes: raw
                .capes
                .into_iter()
                .map(|cape| CapeView { active: cape.state == "ACTIVE", id: cape.id, alias: cape.alias, url: https(&cape.url) })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerSkin {
    pub name: String,
    pub variant: SkinVariant,
    pub png: Vec<u8>,
}

/// Texture URLs are served as plain http by the API but the host supports https.
fn https(url: &str) -> String {
    url.replacen("http://", "https://", 1)
}

pub struct SkinsApi {
    client: Client,
    services: String,
    mojang: String,
    session: String,
    /// Tests serve textures from a local server instead of Mojang's texture host.
    #[cfg(test)]
    texture_base: Option<String>,
}

impl SkinsApi {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            services: SERVICES.into(),
            mojang: MOJANG.into(),
            session: SESSION.into(),
            #[cfg(test)]
            texture_base: None,
        }
    }

    #[cfg(test)]
    pub fn with_base(client: Client, base: &str) -> Self {
        let base = base.trim_end_matches('/').to_owned();
        Self { client, services: base.clone(), mojang: base.clone(), session: base.clone(), texture_base: Some(base) }
    }

    async fn profile_from(&self, request: RequestBuilder) -> AppResult<ProfileView> {
        let response = request.send().await?;
        match response.status() {
            status if status.is_success() => Ok(response.json::<RawProfile>().await?.into()),
            StatusCode::UNAUTHORIZED => Err(crate::error::AuthFailure::SessionExpired.into()),
            StatusCode::BAD_REQUEST => Err(AppError::InvalidInput("Mojang rejected this request".into())),
            StatusCode::TOO_MANY_REQUESTS => Err(AppError::InvalidInput("too many requests, wait a moment and try again".into())),
            status => Err(AppError::HttpStatus { status: status.as_u16(), url: self.services.clone() }),
        }
    }

    pub async fn profile(&self, token: &str) -> AppResult<ProfileView> {
        self.profile_from(self.client.get(format!("{}/minecraft/profile", self.services)).bearer_auth(token)).await
    }

    pub async fn upload_skin(&self, token: &str, png: Vec<u8>, variant: SkinVariant) -> AppResult<ProfileView> {
        validate_png(&png)?;
        let file = multipart::Part::bytes(png)
            .file_name("skin.png")
            .mime_str("image/png")
            .map_err(|error| AppError::Internal(error.to_string()))?;
        let form = multipart::Form::new().text("variant", variant.api_name()).part("file", file);
        self.profile_from(
            self.client
                .post(format!("{}/minecraft/profile/skins", self.services))
                .bearer_auth(token)
                .multipart(form),
        )
        .await
    }

    pub async fn reset_skin(&self, token: &str) -> AppResult<ProfileView> {
        self.profile_from(self.client.delete(format!("{}/minecraft/profile/skins/active", self.services)).bearer_auth(token))
            .await
    }

    /// Selects a cape, or hides the cape when `cape_id` is `None`.
    pub async fn set_cape(&self, token: &str, cape_id: Option<&str>) -> AppResult<ProfileView> {
        let url = format!("{}/minecraft/profile/capes/active", self.services);
        let request = match cape_id {
            Some(id) => self.client.put(url).json(&json!({ "capeId": id })),
            None => self.client.delete(url),
        };
        self.profile_from(request.bearer_auth(token)).await
    }

    pub async fn download_texture(&self, url: &str) -> AppResult<Vec<u8>> {
        let parsed = Url::parse(&https(url)).map_err(|_| AppError::InvalidInput("invalid skin address".into()))?;
        if parsed.scheme() != "https" || parsed.host_str() != Some(TEXTURE_HOST) {
            return Err(AppError::InvalidInput("skins can only be downloaded from Mojang's texture server".into()));
        }
        #[cfg(test)]
        let parsed = match &self.texture_base {
            Some(base) => Url::parse(&format!("{base}{}", parsed.path())).map_err(|error| AppError::Internal(error.to_string()))?,
            None => parsed,
        };
        let url: String = parsed.into();
        let bytes = self.client.get(url).send().await?.error_for_status()?.bytes().await?;
        if bytes.len() > MAX_TEXTURE_BYTES {
            return Err(AppError::InvalidInput("the skin file is too large".into()));
        }
        Ok(bytes.to_vec())
    }

    /// Looks up a player's current skin by username.
    pub async fn player_skin(&self, name: &str) -> AppResult<PlayerSkin> {
        if name.is_empty() || name.len() > 16 || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
            return Err(AppError::InvalidInput("that is not a valid Minecraft username".into()));
        }
        let lookup = self.client.get(format!("{}/users/profiles/minecraft/{name}", self.mojang)).send().await?;
        if matches!(lookup.status(), StatusCode::NOT_FOUND | StatusCode::NO_CONTENT) {
            return Err(AppError::SkinNotFound);
        }
        let found: Value = lookup.error_for_status()?.json().await?;
        let id = found["id"].as_str().filter(|id| id.bytes().all(|byte| byte.is_ascii_hexdigit())).ok_or(AppError::SkinNotFound)?;
        let canonical = found["name"].as_str().unwrap_or(name).to_owned();

        let profile: Value = self
            .client
            .get(format!("{}/session/minecraft/profile/{id}", self.session))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let encoded = profile["properties"]
            .as_array()
            .and_then(|properties| properties.iter().find(|property| property["name"] == "textures"))
            .and_then(|property| property["value"].as_str())
            .ok_or(AppError::SkinNotFound)?;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|error| AppError::Internal(error.to_string()))?;
        let textures: Value = serde_json::from_slice(&decoded)?;
        let skin = &textures["textures"]["SKIN"];
        let url = skin["url"].as_str().ok_or(AppError::SkinNotFound)?;
        let variant = if skin["metadata"]["model"] == "slim" { SkinVariant::Slim } else { SkinVariant::Classic };

        Ok(PlayerSkin { name: canonical, variant, png: self.download_texture(url).await? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skins::library::test_png;
    use crate::test_support::start;

    const PROFILE: &str = r#"{"id":"abc","name":"Steve",
        "skins":[{"id":"1","state":"INACTIVE","url":"http://textures.minecraft.net/texture/old","variant":"CLASSIC"},
                 {"id":"2","state":"ACTIVE","url":"http://textures.minecraft.net/texture/new","variant":"SLIM"}],
        "capes":[{"id":"c1","state":"ACTIVE","url":"http://textures.minecraft.net/texture/cape","alias":"Migrator"}]}"#;

    #[tokio::test]
    async fn reads_the_active_skin_variant_and_capes() {
        let mock = start(vec![("/minecraft/profile", vec![(200, PROFILE)])]).await;

        let profile = SkinsApi::with_base(Client::new(), &mock.base).profile("TOKEN").await.unwrap();

        assert_eq!(profile.name, "Steve");
        assert_eq!(profile.skin_url.as_deref(), Some("https://textures.minecraft.net/texture/new"));
        assert_eq!(profile.variant, SkinVariant::Slim);
        assert!(profile.capes[0].active && profile.capes[0].url.starts_with("https://"));
        assert_eq!(mock.last("/minecraft/profile").authorization.as_deref(), Some("Bearer TOKEN"));
    }

    #[tokio::test]
    async fn uploads_a_skin_as_multipart_with_its_variant() {
        let mock = start(vec![("/minecraft/profile/skins", vec![(200, PROFILE)])]).await;

        SkinsApi::with_base(Client::new(), &mock.base)
            .upload_skin("TOKEN", test_png(64, 64), SkinVariant::Slim)
            .await
            .unwrap();

        let body = mock.last("/minecraft/profile/skins").body;
        assert!(body.contains("name=\"variant\"") && body.contains("slim"));
        assert!(body.contains("name=\"file\"") && body.contains("image/png"));
    }

    #[tokio::test]
    async fn refuses_to_upload_something_that_is_not_a_skin() {
        let mock = start(vec![("/minecraft/profile/skins", vec![(200, PROFILE)])]).await;

        let result = SkinsApi::with_base(Client::new(), &mock.base)
            .upload_skin("TOKEN", b"not a png".to_vec(), SkinVariant::Classic)
            .await;

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
        assert_eq!(mock.count("/minecraft/profile/skins"), 0);
    }

    #[tokio::test]
    async fn selects_and_hides_capes_and_resets_the_skin() {
        let mock = start(vec![
            ("/minecraft/profile/capes/active", vec![(200, PROFILE)]),
            ("/minecraft/profile/skins/active", vec![(200, PROFILE)]),
        ])
        .await;
        let api = SkinsApi::with_base(Client::new(), &mock.base);

        api.set_cape("T", Some("c1")).await.unwrap();
        assert!(mock.last("/minecraft/profile/capes/active").body.contains(r#""capeId":"c1""#));
        api.set_cape("T", None).await.unwrap();
        api.reset_skin("T").await.unwrap();

        assert_eq!(mock.count("/minecraft/profile/capes/active"), 2);
        assert_eq!(mock.count("/minecraft/profile/skins/active"), 1);
    }

    #[tokio::test]
    async fn maps_rejections_to_clear_errors() {
        let mock = start(vec![
            ("/minecraft/profile", vec![(401, "{}")]),
            ("/minecraft/profile/skins/active", vec![(429, "{}")]),
        ])
        .await;
        let api = SkinsApi::with_base(Client::new(), &mock.base);

        assert!(matches!(api.profile("T").await, Err(AppError::Auth(_))));
        assert!(matches!(api.reset_skin("T").await, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn finds_another_players_skin_by_name() {
        let textures = base64::engine::general_purpose::STANDARD.encode(
            r#"{"textures":{"SKIN":{"url":"http://textures.minecraft.net/texture/abc","metadata":{"model":"slim"}}}}"#,
        );
        let session = format!(r#"{{"id":"069a79f4","name":"Notch","properties":[{{"name":"textures","value":"{textures}"}}]}}"#);
        let mock = start(vec![
            ("/users/profiles/minecraft/Notch", vec![(200, r#"{"id":"069a79f4","name":"Notch"}"#.to_owned())]),
            ("/session/minecraft/profile/069a79f4", vec![(200, session)]),
            ("/texture/abc", vec![(200, "PNGDATA".to_owned())]),
        ])
        .await;

        let skin = SkinsApi::with_base(Client::new(), &mock.base).player_skin("Notch").await.unwrap();

        assert_eq!((skin.name.as_str(), skin.variant), ("Notch", SkinVariant::Slim));
        assert_eq!(skin.png, b"PNGDATA");
    }

    #[tokio::test]
    async fn reports_unknown_players() {
        let mock = start(vec![("/users/profiles/minecraft/Nobody", vec![(404, "{}")])]).await;

        let result = SkinsApi::with_base(Client::new(), &mock.base).player_skin("Nobody").await;

        assert!(matches!(result, Err(AppError::SkinNotFound)));
    }
    #[tokio::test]
    async fn rejects_invalid_usernames_and_foreign_texture_hosts_without_network_access() {
        let api = SkinsApi::new(Client::new());
        for name in ["", "way_too_long_for_a_name", "bad name", "../x", "a/b"] {
            assert!(matches!(api.player_skin(name).await, Err(AppError::InvalidInput(_))), "{name:?}");
        }
        for url in ["https://evil.example/skin.png", "http://127.0.0.1/skin.png", "file:///etc/passwd"] {
            assert!(api.download_texture(url).await.is_err(), "{url}");
        }
    }
}
