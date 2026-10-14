use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const ENDPOINT: &str = "https://api.web3forms.com/submit";
const KEY_ENV: &str = "KAIROS_FEEDBACK_KEY";
const BUILT_IN_KEY: Option<&str> = option_env!("KAIROS_FEEDBACK_KEY");
const MIN_MESSAGE: usize = 10;
const MAX_MESSAGE: usize = 4000;
const MAX_CONTACT: usize = 120;
const MAX_DETAILS: usize = 6000;
const MAX_SUBJECT: usize = 60;
/// Seconds before another message can be sent, so a double click or a loop cannot flood the inbox.
const COOLDOWN_SECS: u64 = 20;

static LAST_SENT: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Bug,
    Idea,
    Other,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Idea => "idea",
            Self::Other => "message",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub kind: Kind,
    pub message: String,
    pub contact: Option<String>,
    pub details: Option<String>,
}

fn invalid(message: &str) -> AppError {
    AppError::InvalidInput(message.to_owned())
}

/// The key of the form service this build sends to. Keys are public identifiers, not secrets.
fn access_key() -> Option<String> {
    std::env::var(KEY_ENV)
        .ok()
        .or_else(|| BUILT_IN_KEY.map(str::to_owned))
        .map(|key| key.trim().to_owned())
        .filter(|key| !key.is_empty() && key.len() <= 64 && key.bytes().all(|byte| byte.is_ascii_hexdigit() || byte == b'-'))
}

pub fn is_available() -> bool {
    access_key().is_some()
}

fn clipped(text: &str, limit: usize) -> String {
    text.trim().chars().take(limit).collect()
}

/// Checks and trims what the player typed.
fn clean(report: Report) -> AppResult<Report> {
    let message = report.message.trim().to_owned();
    let length = message.chars().count();
    if length < MIN_MESSAGE {
        return Err(invalid("write a little more, so there is something to act on"));
    }
    if length > MAX_MESSAGE {
        return Err(invalid("the message is too long"));
    }
    Ok(Report {
        kind: report.kind,
        message,
        contact: report.contact.map(|contact| clipped(&contact, MAX_CONTACT)).filter(|contact| !contact.is_empty()),
        details: report.details.map(|details| clipped(&details, MAX_DETAILS)).filter(|details| !details.is_empty()),
    })
}

fn looks_like_email(text: &str) -> bool {
    let mut parts = text.split('@');
    matches!((parts.next(), parts.next(), parts.next()), (Some(name), Some(domain), None) if !name.is_empty() && domain.contains('.') && !text.contains(char::is_whitespace))
}

/// The request body: a subject that tells what it is, and the message with the contact and details below it.
fn body(key: &str, report: &Report, version: &str) -> Value {
    let first_line = report.message.lines().next().unwrap_or_default();
    let mut text = report.message.clone();
    if let Some(contact) = &report.contact {
        text.push_str(&format!("\n\n---\nContact: {contact}"));
    }
    if let Some(details) = &report.details {
        text.push_str(&format!("\n\n---\n{details}"));
    }
    let mut body = json!({
        "access_key": key,
        "subject": format!("[Kairos {version}] {}: {}", report.kind.label(), clipped(first_line, MAX_SUBJECT)),
        "from_name": "Kairos feedback",
        "message": text,
    });
    if let Some(contact) = report.contact.as_deref().filter(|contact| looks_like_email(contact)) {
        body["replyto"] = json!(contact);
    }
    body
}

fn seconds_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs())
}

async fn submit_to(client: &Client, endpoint: &str, key: &str, report: Report, version: &str) -> AppResult<()> {
    let report = clean(report)?;
    let response = client.post(endpoint).header("Accept", "application/json").json(&body(key, &report, version)).send().await?;
    let status = response.status();
    let answer: Value = response.json().await.unwrap_or(Value::Null);
    if status.is_success() && answer["success"].as_bool() == Some(true) {
        return Ok(());
    }
    let reason = answer["message"].as_str().map_or_else(|| format!("the form service answered with {}", status.as_u16()), str::to_owned);
    Err(AppError::InvalidInput(reason))
}

pub async fn submit(client: &Client, report: Report, version: &str) -> AppResult<()> {
    let key = access_key().ok_or_else(|| invalid("this build cannot send feedback by itself"))?;
    let now = seconds_now();
    let last = LAST_SENT.load(Ordering::Relaxed);
    if now.saturating_sub(last) < COOLDOWN_SECS {
        return Err(invalid("you just sent a message, wait a few seconds"));
    }
    submit_to(client, ENDPOINT, &key, report, version).await?;
    LAST_SENT.store(now, Ordering::Relaxed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn report(message: &str) -> Report {
        Report { kind: Kind::Bug, message: message.into(), contact: None, details: None }
    }

    /// A one-shot server that records the request and answers with the given status and body.
    async fn mock(status: &str, answer: &'static str) -> (String, tokio::task::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}/submit", listener.local_addr().unwrap());
        let status = status.to_owned();
        let task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut received = Vec::new();
            let mut buffer = [0u8; 4096];
            loop {
                let read = stream.read(&mut buffer).await.unwrap();
                received.extend_from_slice(&buffer[..read]);
                let text = String::from_utf8_lossy(&received).into_owned();
                if let Some(split) = text.find("\r\n\r\n") {
                    let length = text[..split].to_lowercase().split("content-length:").nth(1).and_then(|rest| rest.split("\r\n").next()).and_then(|value| value.trim().parse::<usize>().ok()).unwrap_or(0);
                    if received.len() >= split + 4 + length {
                        break;
                    }
                }
                if read == 0 {
                    break;
                }
            }
            let response = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}", answer.len());
            stream.write_all(response.as_bytes()).await.unwrap();
            String::from_utf8_lossy(&received).into_owned()
        });
        (address, task)
    }

    #[test]
    fn refuses_messages_with_nothing_in_them_or_far_too_much() {
        assert!(clean(report("short")).is_err());
        assert!(clean(report(&"x".repeat(MAX_MESSAGE + 1))).is_err());
        assert!(clean(report("The game closes when I open the map.")).is_ok());
    }

    #[test]
    fn trims_and_limits_the_optional_parts() {
        let cleaned = clean(Report {
            kind: Kind::Idea,
            message: "   Please add a dark red theme.  ".into(),
            contact: Some("  ".into()),
            details: Some("d".repeat(MAX_DETAILS + 500)),
        })
        .unwrap();

        assert_eq!(cleaned.message, "Please add a dark red theme.");
        assert!(cleaned.contact.is_none());
        assert_eq!(cleaned.details.unwrap().chars().count(), MAX_DETAILS);
    }

    #[test]
    fn writes_a_subject_that_says_what_it_is_and_replies_to_an_email_only() {
        let mut with_email = clean(report("The launcher freezes\nwhen I click Play.")).unwrap();
        with_email.contact = Some("player@example.com".into());
        with_email.details = Some("Kairos 0.1.2".into());

        let value = body("key-1", &with_email, "0.1.2");

        assert_eq!(value["subject"], "[Kairos 0.1.2] bug: The launcher freezes");
        assert_eq!(value["replyto"], "player@example.com");
        let message = value["message"].as_str().unwrap();
        assert!(message.starts_with("The launcher freezes\nwhen I click Play."));
        assert!(message.contains("Contact: player@example.com") && message.ends_with("Kairos 0.1.2"));

        with_email.contact = Some("Steve#1234 on Discord".into());
        assert!(body("key-1", &with_email, "0.1.2").get("replyto").is_none());
    }

    #[test]
    fn only_a_plain_key_is_accepted() {
        // Only this test touches the variable, so parallel tests do not see it change.
        std::env::remove_var(KEY_ENV);
        std::env::set_var(KEY_ENV, "not a key!");
        assert!(access_key().is_none() || BUILT_IN_KEY.is_some());
        std::env::set_var(KEY_ENV, "1a2b3c4d-0000-1111-2222-333344445555");
        assert_eq!(access_key().as_deref(), Some("1a2b3c4d-0000-1111-2222-333344445555"));
        std::env::remove_var(KEY_ENV);
    }

    #[tokio::test]
    async fn sends_the_message_and_accepts_a_success_answer() {
        let (address, request) = mock("200 OK", r#"{"success":true,"message":"Email sent successfully!"}"#).await;

        submit_to(&Client::new(), &address, "key-1", report("Mods do not show up in the list."), "0.1.2").await.unwrap();

        let received = request.await.unwrap();
        assert!(received.starts_with("POST /submit"));
        assert!(received.contains("\"access_key\":\"key-1\""));
        assert!(received.contains("Mods do not show up in the list."));
    }

    #[tokio::test]
    async fn reports_the_reason_when_the_service_says_no() {
        let (address, _) = mock("400 Bad Request", r#"{"success":false,"message":"Invalid access key"}"#).await;

        let error = submit_to(&Client::new(), &address, "bad", report("A message long enough to send."), "0.1.2").await.unwrap_err();

        assert!(error.to_string().contains("Invalid access key"), "{error}");
    }

    #[tokio::test]
    async fn does_not_even_call_the_service_for_an_empty_message() {
        let error = submit_to(&Client::new(), "http://127.0.0.1:9/never", "key", report("hi"), "0.1.2").await.unwrap_err();
        assert!(matches!(error, AppError::InvalidInput(_)));
    }
}
