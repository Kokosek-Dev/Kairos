use crate::error::{AppError, AppResult};
use futures_util::{StreamExt, TryStreamExt};
use reqwest::{header::RANGE, Client, StatusCode};
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

const CONCURRENCY: usize = 16;
const MAX_ATTEMPTS: u32 = 3;

#[derive(Debug, Clone)]
pub enum Checksum {
    Sha1(String),
    Sha256(String),
    Sha512(String),
}

#[derive(Debug, Clone)]
pub struct DownloadItem {
    pub url: String,
    pub path: PathBuf,
    pub checksum: Option<Checksum>,
    pub size: Option<u64>,
}

#[derive(Debug, Default)]
pub struct Counters {
    files: AtomicU64,
    bytes: AtomicU64,
}

impl Counters {
    pub fn files(&self) -> u64 {
        self.files.load(Ordering::Relaxed)
    }

    pub fn bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }
}

pub async fn download_all(
    client: &Client,
    items: Vec<DownloadItem>,
    cancel: &CancellationToken,
    counters: &Arc<Counters>,
) -> AppResult<()> {
    let mut seen = HashSet::new();
    let items: Vec<DownloadItem> = items.into_iter().filter(|item| seen.insert(item.path.clone())).collect();

    futures_util::stream::iter(items)
        .map(|item| async move { ensure_file(client, &item, cancel, counters).await })
        .buffer_unordered(CONCURRENCY)
        .try_collect::<Vec<()>>()
        .await
        .map(drop)
}

pub async fn download_with_progress(
    client: &Client,
    items: Vec<DownloadItem>,
    cancel: &CancellationToken,
    mut on_tick: impl FnMut(&Counters),
) -> AppResult<()> {
    let counters = Arc::new(Counters::default());
    let download = download_all(client, items, cancel, &counters);
    tokio::pin!(download);
    let mut ticker = tokio::time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            result = &mut download => {
                result?;
                on_tick(&counters);
                return Ok(());
            }
            _ = ticker.tick() => on_tick(&counters),
        }
    }
}

async fn ensure_file(
    client: &Client,
    item: &DownloadItem,
    cancel: &CancellationToken,
    counters: &Counters,
) -> AppResult<()> {
    if item.path.components().any(|part| part == Component::ParentDir) {
        return Err(AppError::Internal(format!(
            "refusing to write outside the data directory: {}",
            item.path.display()
        )));
    }

    if is_valid(&item.path, item).await? {
        counters.bytes.fetch_add(item.size.unwrap_or(0), Ordering::Relaxed);
        counters.files.fetch_add(1, Ordering::Relaxed);
        return Ok(());
    }

    let mut last_error = None;
    for _ in 0..MAX_ATTEMPTS {
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        let mut counted = 0;
        match download_once(client, item, cancel, counters, &mut counted).await {
            Ok(()) => {
                counters.files.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
            Err(AppError::Cancelled) => return Err(AppError::Cancelled),
            Err(error) => {
                counters.bytes.fetch_sub(counted, Ordering::Relaxed);
                if matches!(error, AppError::HttpStatus { status: 404, .. }) {
                    return Err(error);
                }
                last_error = Some(error);
            }
        }
    }
    Err(last_error.unwrap_or(AppError::Internal("download failed".into())))
}

async fn download_once(
    client: &Client,
    item: &DownloadItem,
    cancel: &CancellationToken,
    counters: &Counters,
    counted: &mut u64,
) -> AppResult<()> {
    if let Some(parent) = item.path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let part = part_path(&item.path);
    let existing = fs::metadata(&part).await.map(|meta| meta.len()).unwrap_or(0);

    let mut request = client.get(&item.url);
    if existing > 0 {
        request = request.header(RANGE, format!("bytes={existing}-"));
    }
    let response = request.send().await?;

    let append = match response.status() {
        StatusCode::PARTIAL_CONTENT => true,
        StatusCode::OK => false,
        StatusCode::RANGE_NOT_SATISFIABLE => {
            let _ = fs::remove_file(&part).await;
            return Err(AppError::Internal("stale partial download was reset".into()));
        }
        status => {
            return Err(AppError::HttpStatus {
                status: status.as_u16(),
                url: item.url.clone(),
            })
        }
    };

    let mut file = if append {
        counters.bytes.fetch_add(existing, Ordering::Relaxed);
        *counted += existing;
        OpenOptions::new().append(true).open(&part).await?
    } else {
        File::create(&part).await?
    };

    let mut stream = response.bytes_stream();
    loop {
        let chunk = tokio::select! {
            _ = cancel.cancelled() => {
                file.flush().await?;
                return Err(AppError::Cancelled);
            }
            chunk = stream.next() => chunk,
        };
        let Some(chunk) = chunk else { break };
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        counters.bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed);
        *counted += chunk.len() as u64;
    }
    file.flush().await?;
    drop(file);

    if !is_valid(&part, item).await? {
        let _ = fs::remove_file(&part).await;
        return Err(AppError::Checksum(item.url.clone()));
    }
    fs::rename(&part, &item.path).await?;
    Ok(())
}

fn part_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    path.with_file_name(name)
}

async fn is_valid(path: &Path, item: &DownloadItem) -> AppResult<bool> {
    let Ok(meta) = fs::metadata(path).await else {
        return Ok(false);
    };
    if item.size.is_some_and(|size| size != meta.len()) {
        return Ok(false);
    }
    match &item.checksum {
        Some(checksum) => matches_checksum(path, checksum).await,
        None => Ok(true),
    }
}

async fn matches_checksum(path: &Path, checksum: &Checksum) -> AppResult<bool> {
    let (actual, expected) = match checksum {
        Checksum::Sha1(expected) => (hash_file::<Sha1>(path).await?, expected),
        Checksum::Sha256(expected) => (hash_file::<Sha256>(path).await?, expected),
        Checksum::Sha512(expected) => (hash_file::<Sha512>(path).await?, expected),
    };
    Ok(actual.eq_ignore_ascii_case(expected))
}

pub fn sha1_bytes(data: &[u8]) -> String {
    hex::encode(Sha1::digest(data))
}

async fn hash_file<D: Digest>(path: &Path) -> AppResult<String> {
    let mut file = File::open(path).await?;
    let mut hasher = D::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tokio::net::TcpListener;

    fn payload() -> Vec<u8> {
        (0..1_000_000u32).map(|n| (n % 251) as u8).collect()
    }

    async fn serve(body: Vec<u8>, ranges: Arc<Mutex<Vec<u64>>>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else { return };
                let body = body.clone();
                let ranges = ranges.clone();
                tokio::spawn(async move {
                    let mut request = Vec::new();
                    let mut buffer = [0u8; 1024];
                    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                        let read = socket.read(&mut buffer).await.unwrap_or(0);
                        if read == 0 {
                            return;
                        }
                        request.extend_from_slice(&buffer[..read]);
                    }
                    let text = String::from_utf8_lossy(&request).to_lowercase();
                    let start = text
                        .split("range: bytes=")
                        .nth(1)
                        .and_then(|rest| rest.split('-').next())
                        .and_then(|value| value.trim().parse::<usize>().ok());
                    let (status, slice) = match start {
                        Some(start) => {
                            ranges.lock().unwrap().push(start as u64);
                            ("206 Partial Content", &body[start..])
                        }
                        None => ("200 OK", &body[..]),
                    };
                    let head = format!(
                        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        slice.len()
                    );
                    let _ = socket.write_all(head.as_bytes()).await;
                    let _ = socket.write_all(slice).await;
                });
            }
        });
        format!("http://{address}/file.bin")
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("zenith-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn item(url: String, path: PathBuf, body: &[u8]) -> DownloadItem {
        DownloadItem {
            url,
            path,
            checksum: Some(Checksum::Sha1(sha1_bytes(body))),
            size: Some(body.len() as u64),
        }
    }

    async fn run(item: DownloadItem) -> (AppResult<()>, Arc<Counters>) {
        let counters = Arc::new(Counters::default());
        let result = download_all(&Client::new(), vec![item], &CancellationToken::new(), &counters).await;
        (result, counters)
    }

    #[tokio::test]
    async fn downloads_and_verifies_a_file() {
        let body = payload();
        let url = serve(body.clone(), Arc::default()).await;
        let dir = scratch("fresh");
        let target = dir.join("nested").join("file.bin");

        let (result, counters) = run(item(url, target.clone(), &body)).await;

        result.unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), body);
        assert_eq!((counters.files(), counters.bytes()), (1, body.len() as u64));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn resumes_from_a_partial_file_with_a_range_request() {
        let body = payload();
        let ranges: Arc<Mutex<Vec<u64>>> = Arc::default();
        let url = serve(body.clone(), ranges.clone()).await;
        let dir = scratch("resume");
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("file.bin");
        std::fs::write(part_path(&target), &body[..300_000]).unwrap();

        let (result, counters) = run(item(url, target.clone(), &body)).await;

        result.unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), body);
        assert_eq!(*ranges.lock().unwrap(), vec![300_000]);
        assert_eq!(counters.bytes(), body.len() as u64);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn recovers_from_a_corrupted_partial_file() {
        let body = payload();
        let url = serve(body.clone(), Arc::default()).await;
        let dir = scratch("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("file.bin");
        std::fs::write(part_path(&target), vec![0u8; 300_000]).unwrap();

        let (result, counters) = run(item(url, target.clone(), &body)).await;

        result.unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), body);
        assert_eq!(counters.bytes(), body.len() as u64);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn rejects_a_file_with_the_wrong_checksum() {
        let body = payload();
        let url = serve(body.clone(), Arc::default()).await;
        let dir = scratch("badsum");
        let target = dir.join("file.bin");
        let mut wrong = item(url, target.clone(), &body);
        wrong.checksum = Some(Checksum::Sha1("0".repeat(40)));

        let (result, counters) = run(wrong).await;

        assert!(matches!(result, Err(AppError::Checksum(_))));
        assert!(!target.exists());
        assert_eq!(counters.bytes(), 0);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn downloads_a_file_listed_twice_only_once() {
        let body = payload();
        let url = serve(body.clone(), Arc::default()).await;
        let dir = scratch("duplicate");
        let target = dir.join("file.bin");
        let duplicate = item(url, target.clone(), &body);
        let counters = Arc::new(Counters::default());

        download_all(&Client::new(), vec![duplicate.clone(), duplicate], &CancellationToken::new(), &counters)
            .await
            .unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), body);
        assert_eq!((counters.files(), counters.bytes()), (1, body.len() as u64));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn skips_files_that_are_already_valid() {
        let body = payload();
        let ranges: Arc<Mutex<Vec<u64>>> = Arc::default();
        let dir = scratch("skip");
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("file.bin");
        std::fs::write(&target, &body).unwrap();

        let (result, counters) = run(item("http://127.0.0.1:1/unreachable".into(), target, &body)).await;

        result.unwrap();
        assert_eq!((counters.files(), counters.bytes()), (1, body.len() as u64));
        assert!(ranges.lock().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn refuses_paths_that_escape_the_data_directory() {
        let body = payload();
        let escape = PathBuf::from("data").join("..").join("evil.bin");

        let (result, _) = run(item("http://127.0.0.1:1/x".into(), escape, &body)).await;

        assert!(matches!(result, Err(AppError::Internal(_))));
    }
}
