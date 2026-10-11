use crate::error::{AppError, AppResult, AuthFailure};
use reqwest::Url;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

const SIGN_IN_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_REQUEST_BYTES: usize = 16 * 1024;

const SUCCESS_PAGE: &str = "<h2>Kairos</h2><p>Signed in. You can close this tab and return to the launcher.</p><p>Zalogowano. Możesz zamknąć tę kartę i wrócić do launchera.</p>";
const FAILURE_PAGE: &str = "<h2>Kairos</h2><p>Sign-in failed. Return to the launcher for details.</p><p>Logowanie nie powiodło się. Wróć do launchera, aby zobaczyć szczegóły.</p>";

pub struct Loopback {
    listener: TcpListener,
    pub redirect_uri: String,
}

/// Starts a one-shot HTTP listener on a free loopback port. Microsoft ignores the port when
/// matching a registered `http://localhost` redirect URI.
pub async fn bind() -> AppResult<Loopback> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    Ok(Loopback { listener, redirect_uri: format!("http://localhost:{port}") })
}

enum Callback {
    Code(String),
    Failed(AppError),
    Ignored,
}

impl Loopback {
    pub async fn wait_for_code(self, expected_state: &str, cancel: &CancellationToken) -> AppResult<String> {
        let deadline = Instant::now() + SIGN_IN_TIMEOUT;
        loop {
            let accepted = tokio::select! {
                _ = cancel.cancelled() => return Err(AppError::Cancelled),
                _ = tokio::time::sleep_until(deadline) => return Err(AuthFailure::Expired.into()),
                accepted = self.listener.accept() => accepted,
            };
            let Ok((stream, _)) = accepted else { continue };
            match handle(stream, expected_state).await {
                Callback::Code(code) => return Ok(code),
                Callback::Failed(error) => return Err(error),
                Callback::Ignored => {}
            }
        }
    }
}

async fn read_request_line(stream: &mut TcpStream) -> Option<String> {
    let mut data = Vec::new();
    let mut buffer = [0u8; 2048];
    let read_all = async {
        while !data.windows(2).any(|window| window == b"\r\n") && data.len() < MAX_REQUEST_BYTES {
            let read = stream.read(&mut buffer).await.ok()?;
            if read == 0 {
                return None;
            }
            data.extend_from_slice(&buffer[..read]);
        }
        Some(())
    };
    tokio::time::timeout(REQUEST_READ_TIMEOUT, read_all).await.ok()??;
    let line_end = data.windows(2).position(|window| window == b"\r\n")?;
    Some(String::from_utf8_lossy(&data[..line_end]).into_owned())
}

async fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let page = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Kairos</title><body style=\"font-family:system-ui;background:#0b1020;color:#e8eefc;display:grid;place-items:center;height:100vh;margin:0;text-align:center\"><div>{body}</div></body>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{page}",
        page.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
}

async fn handle(mut stream: TcpStream, expected_state: &str) -> Callback {
    let Some(line) = read_request_line(&mut stream).await else {
        return Callback::Ignored;
    };
    let target = line.split_whitespace().nth(1).unwrap_or_default();
    let Ok(url) = Url::parse(&format!("http://localhost{target}")) else {
        respond(&mut stream, "400 Bad Request", FAILURE_PAGE).await;
        return Callback::Ignored;
    };
    let param = |name: &str| url.query_pairs().find(|(key, _)| key == name).map(|(_, value)| value.into_owned());

    if url.path() != "/" || (param("code").is_none() && param("error").is_none()) {
        respond(&mut stream, "404 Not Found", FAILURE_PAGE).await;
        return Callback::Ignored;
    }
    if param("state").as_deref() != Some(expected_state) {
        respond(&mut stream, "400 Bad Request", FAILURE_PAGE).await;
        return Callback::Ignored;
    }
    if let Some(error) = param("error") {
        respond(&mut stream, "200 OK", FAILURE_PAGE).await;
        return Callback::Failed(match error.as_str() {
            "access_denied" => AuthFailure::Declined.into(),
            other => AppError::Internal(format!("Microsoft sign-in failed: {other}")),
        });
    }
    match param("code") {
        Some(code) => {
            respond(&mut stream, "200 OK", SUCCESS_PAGE).await;
            Callback::Code(code)
        }
        None => Callback::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn browse(redirect_uri: &str, query: &str) -> reqwest::StatusCode {
        reqwest::get(format!("{redirect_uri}/{query}")).await.unwrap().status()
    }

    #[tokio::test]
    async fn returns_the_code_when_the_browser_comes_back_with_the_right_state() {
        let loopback = bind().await.unwrap();
        let redirect = loopback.redirect_uri.clone();
        let waiting = tokio::spawn(async move { loopback.wait_for_code("STATE", &CancellationToken::new()).await });

        assert_eq!(browse(&redirect, "?code=THE-CODE&state=STATE").await, 200);

        assert_eq!(waiting.await.unwrap().unwrap(), "THE-CODE");
    }

    #[tokio::test]
    async fn ignores_favicon_requests_and_forged_states() {
        let loopback = bind().await.unwrap();
        let redirect = loopback.redirect_uri.clone();
        let waiting = tokio::spawn(async move { loopback.wait_for_code("STATE", &CancellationToken::new()).await });

        assert_eq!(reqwest::get(format!("{redirect}/favicon.ico")).await.unwrap().status(), 404);
        assert_eq!(browse(&redirect, "?code=EVIL&state=OTHER").await, 400);
        assert_eq!(browse(&redirect, "?code=GOOD&state=STATE").await, 200);

        assert_eq!(waiting.await.unwrap().unwrap(), "GOOD");
    }

    #[tokio::test]
    async fn reports_a_declined_sign_in() {
        let loopback = bind().await.unwrap();
        let redirect = loopback.redirect_uri.clone();
        let waiting = tokio::spawn(async move { loopback.wait_for_code("STATE", &CancellationToken::new()).await });

        browse(&redirect, "?error=access_denied&state=STATE").await;

        assert!(matches!(waiting.await.unwrap(), Err(AppError::Auth(AuthFailure::Declined))));
    }

    #[tokio::test]
    async fn stops_waiting_when_cancelled() {
        let loopback = bind().await.unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();

        assert!(matches!(loopback.wait_for_code("STATE", &cancel).await, Err(AppError::Cancelled)));
    }
}
