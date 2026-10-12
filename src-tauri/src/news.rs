use crate::error::{AppError, AppResult};
use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const URL_ENV: &str = "KAIROS_NEWS_URL";
const BUILT_IN_URL: Option<&str> = option_env!("KAIROS_NEWS_URL");
const MAX_BYTES: usize = 64 * 1024;
const MAX_ITEMS: usize = 12;
const MAX_ID: usize = 64;
const MAX_TITLE: usize = 120;
const MAX_BODY: usize = 400;

/// A text that is either one string for everybody or one string per language code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Text {
    Plain(String),
    ByLanguage(BTreeMap<String, String>),
}

impl Text {
    fn clamped(self, limit: usize) -> Option<Self> {
        let cut = |value: String| value.chars().take(limit).collect::<String>();
        match self {
            Self::Plain(value) => Some(Self::Plain(cut(value))).filter(|text| !text.is_blank()),
            Self::ByLanguage(values) => {
                let values: BTreeMap<_, _> = values.into_iter().map(|(language, value)| (language, cut(value))).collect();
                Some(Self::ByLanguage(values)).filter(|text| !text.is_blank())
            }
        }
    }

    fn is_blank(&self) -> bool {
        match self {
            Self::Plain(value) => value.trim().is_empty(),
            Self::ByLanguage(values) => values.values().all(|value| value.trim().is_empty()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    News,
    Sponsored,
}

#[derive(Debug, Deserialize)]
struct Raw {
    id: String,
    #[serde(default)]
    kind: Kind,
    date: Option<String>,
    title: Text,
    body: Text,
    url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    pub id: String,
    pub kind: Kind,
    pub date: Option<String>,
    pub title: Text,
    pub body: Text,
    pub url: Option<String>,
}

fn news_url() -> Option<String> {
    std::env::var(URL_ENV)
        .ok()
        .or_else(|| BUILT_IN_URL.map(str::to_owned))
        .filter(|url| url.starts_with("https://"))
}

fn web_link(value: &str) -> Option<String> {
    reqwest::Url::parse(value).ok().filter(|url| url.scheme() == "https").map(|url| url.to_string())
}

/// Turns the published list into items that are safe to show; anything malformed is skipped, not trusted.
pub fn parse(bytes: &[u8]) -> AppResult<Vec<NewsItem>> {
    if bytes.len() > MAX_BYTES {
        return Err(AppError::InvalidInput("the news list is too large".into()));
    }
    let raw: Vec<serde_json::Value> = serde_json::from_slice(bytes)?;
    Ok(raw
        .into_iter()
        .filter_map(|value| serde_json::from_value::<Raw>(value).ok())
        .filter(|item| !item.id.is_empty() && item.id.chars().count() <= MAX_ID)
        .filter_map(|item| {
            Some(NewsItem {
                id: item.id,
                kind: item.kind,
                date: item.date.filter(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()),
                title: item.title.clamped(MAX_TITLE)?,
                body: item.body.clamped(MAX_BODY)?,
                url: item.url.as_deref().and_then(web_link),
            })
        })
        .take(MAX_ITEMS)
        .collect())
}

/// The published news, or nothing when no address is configured for this build.
pub async fn fetch(client: &Client) -> AppResult<Vec<NewsItem>> {
    let Some(url) = news_url() else {
        return Ok(Vec::new());
    };
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        return Err(AppError::HttpStatus { status: response.status().as_u16(), url });
    }
    if response.content_length().is_some_and(|length| length > MAX_BYTES as u64) {
        return Err(AppError::InvalidInput("the news list is too large".into()));
    }
    parse(&response.bytes().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_plain_and_per_language_texts() {
        let items = parse(
            br#"[
                {"id":"a","date":"2026-10-12","title":"Hello","body":"World","url":"https://example.com/a"},
                {"id":"b","kind":"sponsored","title":{"en":"Ad","pl":"Reklama"},"body":{"en":"Buy"}}
            ]"#,
        )
        .unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].kind, Kind::News);
        assert_eq!(items[0].url.as_deref(), Some("https://example.com/a"));
        assert_eq!(items[1].kind, Kind::Sponsored);
        assert!(matches!(&items[1].title, Text::ByLanguage(values) if values["pl"] == "Reklama"));
    }

    #[test]
    fn drops_bad_links_dates_and_entries_but_keeps_the_rest() {
        let items = parse(
            br#"[
                {"id":"a","date":"yesterday","title":"T","body":"B","url":"javascript:alert(1)"},
                {"id":"","title":"No id","body":"B"},
                {"id":"c","title":"  ","body":"Blank title"},
                {"id":"d","title":"T","body":"B","url":"http://insecure.example"},
                "not an object"
            ]"#,
        )
        .unwrap();

        let ids: Vec<_> = items.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, ["a", "d"]);
        assert!(items.iter().all(|item| item.url.is_none() && item.date.is_none()));
    }

    #[test]
    fn limits_how_much_is_taken() {
        let long = "x".repeat(MAX_BODY + 50);
        let list: Vec<_> = (0..MAX_ITEMS + 5)
            .map(|index| serde_json::json!({ "id": index.to_string(), "title": "T", "body": long }))
            .collect();
        let items = parse(&serde_json::to_vec(&list).unwrap()).unwrap();

        assert_eq!(items.len(), MAX_ITEMS);
        assert!(matches!(&items[0].body, Text::Plain(body) if body.chars().count() == MAX_BODY));
    }

    #[test]
    fn rejects_anything_that_is_not_a_list_or_is_huge() {
        assert!(parse(br#"{"id":"a"}"#).is_err());
        assert!(parse(&vec![b' '; MAX_BYTES + 1]).is_err());
    }
}
