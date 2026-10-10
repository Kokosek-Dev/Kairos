//! A tiny scripted HTTP server for tests: routes map a path to a queue of canned responses
//! (the last one repeats) and every request is recorded.
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Clone, Debug)]
pub struct Recorded {
    pub path: String,
    pub query: String,
    pub body: String,
    pub authorization: Option<String>,
    pub user_agent: Option<String>,
}

pub struct Mock {
    pub base: String,
    pub requests: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    pub fn count(&self, path: &str) -> usize {
        self.requests.lock().unwrap().iter().filter(|request| request.path == path).count()
    }

    pub fn last(&self, path: &str) -> Recorded {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|request| request.path == path)
            .cloned()
            .unwrap_or_else(|| panic!("no request to {path}"))
    }
}

pub async fn start<S: AsRef<str>>(routes: Vec<(&'static str, Vec<(u16, S)>)>) -> Mock {
    let responses: HashMap<&'static str, VecDeque<(u16, String)>> = routes
        .into_iter()
        .map(|(path, queue)| (path, queue.into_iter().map(|(status, body)| (status, body.as_ref().to_owned())).collect()))
        .collect();
    let responses = Arc::new(Mutex::new(responses));
    let requests: Arc<Mutex<Vec<Recorded>>> = Arc::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());

    let log = requests.clone();
    let own_base = base.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else { return };
            let (responses, log, own_base) = (responses.clone(), log.clone(), own_base.clone());
            tokio::spawn(async move {
                let mut data = Vec::new();
                let mut buffer = [0u8; 4096];
                let (head_end, length) = loop {
                    let read = socket.read(&mut buffer).await.unwrap_or(0);
                    if read == 0 {
                        return;
                    }
                    data.extend_from_slice(&buffer[..read]);
                    if let Some(end) = data.windows(4).position(|window| window == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&data[..end]).to_lowercase();
                        let length = head
                            .lines()
                            .find_map(|line| line.strip_prefix("content-length:"))
                            .and_then(|value| value.trim().parse::<usize>().ok())
                            .unwrap_or(0);
                        break (end + 4, length);
                    }
                };
                while data.len() < head_end + length {
                    let read = socket.read(&mut buffer).await.unwrap_or(0);
                    if read == 0 {
                        break;
                    }
                    data.extend_from_slice(&buffer[..read]);
                }

                let head = String::from_utf8_lossy(&data[..head_end]).into_owned();
                let mut first = head.split_whitespace();
                first.next();
                let target = first.next().unwrap_or_default().to_owned();
                let (path, query) = target.split_once('?').map_or((target.clone(), String::new()), |(path, query)| (path.to_owned(), query.to_owned()));
                let header = |name: &str| {
                    head.lines()
                        .find(|line| line.to_lowercase().starts_with(&format!("{name}:")))
                        .map(|line| line.split_once(':').unwrap().1.trim().to_owned())
                };
                let body = String::from_utf8_lossy(&data[head_end..]).into_owned();
                log.lock().unwrap().push(Recorded {
                    path: path.clone(),
                    query,
                    body,
                    authorization: header("authorization"),
                    user_agent: header("user-agent"),
                });

                let (status, body) = {
                    let mut responses = responses.lock().unwrap();
                    match responses.get_mut(path.as_str()) {
                        Some(queue) if queue.len() > 1 => queue.pop_front().unwrap(),
                        Some(queue) => queue.front().cloned().unwrap_or((404, "{}".into())),
                        None => (404, "{}".into()),
                    }
                };
                let body = body.replace("{base}", &own_base);
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    });
    Mock { base, requests }
}