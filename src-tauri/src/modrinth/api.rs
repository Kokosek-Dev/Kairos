use super::types::{Category, Project, ProjectType, SearchResults, TeamMember, Version};
use crate::error::{AppError, AppResult};
use reqwest::header::USER_AGENT;
use reqwest::{Client, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

const API_BASE: &str = "https://api.modrinth.com/v2";
const MAX_PAGE_SIZE: u32 = 50;

/// The content service asks for a uniquely identifying `User-Agent`: `<owner>/<project>/<version>`.
const USER_AGENT_VALUE: &str = concat!("zenith-launcher/zenith/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub query: String,
    pub project_type: Option<ProjectType>,
    pub game_version: Option<String>,
    pub loader: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub sort: Option<String>,
    #[serde(default)]
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub struct VersionFilter {
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
}

pub struct ModrinthApi {
    client: Client,
    base: String,
}

pub fn build_facets(query: &SearchQuery) -> Vec<Vec<String>> {
    let mut facets = Vec::new();
    if let Some(kind) = query.project_type {
        facets.push(vec![format!("project_type:{}", kind.slug())]);
    }
    if let Some(version) = &query.game_version {
        facets.push(vec![format!("versions:{version}")]);
    }
    if let Some(loader) = &query.loader {
        facets.push(vec![format!("categories:{loader}")]);
    }
    for category in &query.categories {
        facets.push(vec![format!("categories:{category}")]);
    }
    facets
}

fn json_list(values: &[String]) -> AppResult<String> {
    Ok(serde_json::to_string(values)?)
}

impl ModrinthApi {
    pub fn new(client: Client) -> Self {
        Self::with_base(client, API_BASE)
    }

    pub fn with_base(client: Client, base: &str) -> Self {
        Self { client, base: base.trim_end_matches('/').to_owned() }
    }

    fn url(&self, path: &str, query: &[(&str, String)]) -> AppResult<Url> {
        let mut url = Url::parse(&format!("{}{path}", self.base)).map_err(|error| AppError::Internal(error.to_string()))?;
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query.iter().map(|(key, value)| (*key, value.as_str())));
        }
        Ok(url)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, String)]) -> AppResult<T> {
        let response = self
            .client
            .get(self.url(path, query)?)
            .header(USER_AGENT, USER_AGENT_VALUE)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(AppError::HttpStatus { status: response.status().as_u16(), url: path.to_owned() });
        }
        Ok(response.json().await?)
    }

    async fn post<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> AppResult<T> {
        let response = self
            .client
            .post(self.url(path, &[])?)
            .header(USER_AGENT, USER_AGENT_VALUE)
            .json(body)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(AppError::HttpStatus { status: response.status().as_u16(), url: path.to_owned() });
        }
        Ok(response.json().await?)
    }

    /// Downloads a file from Modrinth's CDN (or another trusted host) with the same identity.
    pub fn download_client(&self) -> &Client {
        &self.client
    }

    pub async fn search(&self, query: &SearchQuery) -> AppResult<SearchResults> {
        let facets = serde_json::to_string(&build_facets(query))?;
        let params = [
            ("query", query.query.clone()),
            ("facets", facets),
            ("index", query.sort.clone().unwrap_or_else(|| "relevance".into())),
            ("offset", query.offset.to_string()),
            ("limit", MAX_PAGE_SIZE.to_string()),
        ];
        self.get("/search", &params).await
    }

    pub async fn project(&self, id: &str) -> AppResult<Project> {
        self.get(&format!("/project/{}", encode(id)), &[]).await
    }

    pub async fn projects(&self, ids: &[String]) -> AppResult<Vec<Project>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        self.get("/projects", &[("ids", json_list(ids)?)]).await
    }

    pub async fn members(&self, id: &str) -> AppResult<Vec<TeamMember>> {
        self.get(&format!("/project/{}/members", encode(id)), &[]).await
    }

    pub async fn categories(&self) -> AppResult<Vec<Category>> {
        self.get("/tag/category", &[]).await
    }

    pub async fn versions(&self, project_id: &str, filter: &VersionFilter) -> AppResult<Vec<Version>> {
        let mut params = Vec::new();
        if !filter.loaders.is_empty() {
            params.push(("loaders", json_list(&filter.loaders)?));
        }
        if !filter.game_versions.is_empty() {
            params.push(("game_versions", json_list(&filter.game_versions)?));
        }
        self.get(&format!("/project/{}/version", encode(project_id)), &params).await
    }

    pub async fn version(&self, id: &str) -> AppResult<Version> {
        self.get(&format!("/version/{}", encode(id)), &[]).await
    }

    /// Identifies files by hash, returning the version each file belongs to.
    pub async fn versions_from_hashes(&self, hashes: &[String], algorithm: &str) -> AppResult<HashMap<String, Version>> {
        if hashes.is_empty() {
            return Ok(HashMap::new());
        }
        self.post("/version_files", &serde_json::json!({ "hashes": hashes, "algorithm": algorithm })).await
    }

    /// Finds the newest version for every given file hash that fits the loaders and game versions.
    pub async fn latest_from_hashes(
        &self,
        hashes: &[String],
        algorithm: &str,
        filter: &VersionFilter,
    ) -> AppResult<HashMap<String, Version>> {
        if hashes.is_empty() {
            return Ok(HashMap::new());
        }
        self.post(
            "/version_files/update",
            &serde_json::json!({
                "hashes": hashes,
                "algorithm": algorithm,
                "loaders": filter.loaders,
                "game_versions": filter.game_versions,
            }),
        )
        .await
    }
}

fn encode(segment: &str) -> String {
    Url::parse("http://x/")
        .ok()
        .and_then(|mut url| {
            url.path_segments_mut().ok()?.push(segment);
            Some(url.path().trim_start_matches('/').to_owned())
        })
        .unwrap_or_else(|| segment.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::start;

    fn api(mock: &crate::test_support::Mock) -> ModrinthApi {
        ModrinthApi::with_base(Client::new(), &mock.base)
    }

    #[test]
    fn builds_facets_for_every_filter() {
        let query = SearchQuery {
            query: "sodium".into(),
            project_type: Some(ProjectType::Mod),
            game_version: Some("1.21.1".into()),
            loader: Some("fabric".into()),
            categories: vec!["optimization".into()],
            sort: None,
            offset: 0,
        };

        assert_eq!(
            serde_json::to_string(&build_facets(&query)).unwrap(),
            r#"[["project_type:mod"],["versions:1.21.1"],["categories:fabric"],["categories:optimization"]]"#
        );
        assert!(build_facets(&SearchQuery::default()).is_empty());
    }

    #[tokio::test]
    async fn searches_with_facets_sorting_and_the_identifying_user_agent() {
        let body = r#"{"hits":[{"project_id":"AANobbMI","slug":"sodium","project_type":"mod","title":"Sodium",
            "description":"fast","author":"jellysquid3","icon_url":null,"downloads":5,"follows":2,
            "display_categories":["fabric"],"date_modified":"2026-01-01T00:00:00Z"}],"offset":0,"limit":50,"total_hits":1}"#;
        let mock = start(vec![("/search", vec![(200, body)])]).await;
        let query = SearchQuery {
            query: "sodium".into(),
            project_type: Some(ProjectType::Mod),
            loader: Some("fabric".into()),
            sort: Some("downloads".into()),
            ..SearchQuery::default()
        };

        let results = api(&mock).search(&query).await.unwrap();

        assert_eq!(results.total_hits, 1);
        assert_eq!(results.hits[0].title, "Sodium");
        let request = mock.last("/search");
        assert!(request.query.contains("index=downloads"));
        assert!(request.query.contains("query=sodium"));
        assert!(request.query.contains("project_type%3Amod"));
        assert_eq!(request.user_agent.as_deref(), Some(USER_AGENT_VALUE));
    }

    #[tokio::test]
    async fn requests_versions_with_json_encoded_filters() {
        let mock = start(vec![("/project/sodium/version", vec![(200, "[]")])]).await;
        let filter = VersionFilter { loaders: vec!["fabric".into()], game_versions: vec!["1.21.1".into()] };

        let versions = api(&mock).versions("sodium", &filter).await.unwrap();

        assert!(versions.is_empty());
        let query = mock.last("/project/sodium/version").query;
        assert!(query.contains("loaders=%5B%22fabric%22%5D"));
        assert!(query.contains("game_versions=%5B%221.21.1%22%5D"));
    }

    #[tokio::test]
    async fn rejects_error_statuses_instead_of_parsing_them() {
        let mock = start(vec![("/project/missing", vec![(404, r#"{"error":"not_found"}"#)])]).await;

        let result = api(&mock).project("missing").await;

        assert!(matches!(result, Err(AppError::HttpStatus { status: 404, .. })));
    }
}
