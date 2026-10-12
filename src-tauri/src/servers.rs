use crate::error::{AppError, AppResult};
use crate::paths::DataPaths;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const DEFAULT_PORT: u16 = 25565;
const TIMEOUT: Duration = Duration::from_secs(5);
const MAX_SERVERS: usize = 100;
const MAX_NAME: usize = 48;
const MAX_ADDRESS: usize = 253;
const MAX_RESPONSE: usize = 512 * 1024;
const MAX_FAVICON: usize = 100 * 1024;
const MAX_MOTD: usize = 200;
/// The protocol number sent in the handshake. Servers answer a status request whatever it is.
const HANDSHAKE_PROTOCOL: i32 = 765;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerEntry {
    pub id: String,
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub motd: String,
    pub version: String,
    pub players_online: u32,
    pub players_max: u32,
    pub latency_ms: Option<u64>,
    pub favicon: Option<String>,
}

pub fn load(paths: &DataPaths) -> AppResult<Vec<ServerEntry>> {
    match fs::read(paths.servers_file()) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes).unwrap_or_default()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

fn save(paths: &DataPaths, servers: &[ServerEntry]) -> AppResult<()> {
    let target = paths.servers_file();
    let temp = target.with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(servers)?)?;
    fs::rename(temp, target)?;
    Ok(())
}

/// Splits `host` or `host:port`; anything that is not a plain host name or address is refused.
pub fn parse_address(address: &str) -> AppResult<(String, u16)> {
    let invalid = || AppError::InvalidInput("invalid server address".into());
    let address = address.trim();
    if address.is_empty() || address.len() > MAX_ADDRESS {
        return Err(invalid());
    }
    let (host, port) = match address.rsplit_once(':') {
        Some((host, port)) => (host, port.parse::<u16>().ok().filter(|port| *port > 0).ok_or_else(invalid)?),
        None => (address, DEFAULT_PORT),
    };
    let valid_host = !host.is_empty()
        && !host.starts_with(['.', '-'])
        && !host.ends_with(['.', '-'])
        && host.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-');
    if valid_host {
        Ok((host.to_ascii_lowercase(), port))
    } else {
        Err(invalid())
    }
}

pub fn add(paths: &DataPaths, name: &str, address: &str) -> AppResult<ServerEntry> {
    let (host, port) = parse_address(address)?;
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME {
        return Err(AppError::InvalidInput("invalid server name".into()));
    }
    let mut servers = load(paths)?;
    if servers.len() >= MAX_SERVERS {
        return Err(AppError::InvalidInput("too many servers".into()));
    }
    let entry = ServerEntry {
        id: format!("{:x}", chrono::Utc::now().timestamp_millis()),
        name: name.to_owned(),
        address: if port == DEFAULT_PORT { host } else { format!("{host}:{port}") },
    };
    servers.push(entry.clone());
    save(paths, &servers)?;
    Ok(entry)
}

pub fn remove(paths: &DataPaths, id: &str) -> AppResult<()> {
    let mut servers = load(paths)?;
    servers.retain(|server| server.id != id);
    save(paths, &servers)
}

fn write_varint(buffer: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            buffer.push(byte);
            return;
        }
        buffer.push(byte | 0x80);
    }
}

async fn read_varint(stream: &mut TcpStream) -> AppResult<i32> {
    let mut result = 0u32;
    for shift in (0..35).step_by(7) {
        let byte = stream.read_u8().await?;
        result |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(result as i32);
        }
    }
    Err(AppError::InvalidInput("the server sent an invalid number".into()))
}

fn take_varint(bytes: &[u8], cursor: &mut usize) -> AppResult<i32> {
    let mut result = 0u32;
    for shift in (0..35).step_by(7) {
        let byte = *bytes.get(*cursor).ok_or_else(|| AppError::InvalidInput("the server sent an unexpected answer".into()))?;
        *cursor += 1;
        result |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(result as i32);
        }
    }
    Err(AppError::InvalidInput("the server sent an invalid number".into()))
}

fn packet(id: i32, body: &[u8]) -> Vec<u8> {
    let mut content = Vec::new();
    write_varint(&mut content, id);
    content.extend_from_slice(body);
    let mut framed = Vec::new();
    write_varint(&mut framed, content.len() as i32);
    framed.extend(content);
    framed
}

fn handshake(host: &str, port: u16) -> Vec<u8> {
    let mut body = Vec::new();
    write_varint(&mut body, HANDSHAKE_PROTOCOL);
    write_varint(&mut body, host.len() as i32);
    body.extend_from_slice(host.as_bytes());
    body.extend_from_slice(&port.to_be_bytes());
    write_varint(&mut body, 1);
    packet(0, &body)
}

/// The visible text of a chat component, without formatting codes.
fn flatten(value: &Value, out: &mut String) {
    match value {
        Value::String(text) => {
            let mut chars = text.chars();
            while let Some(character) = chars.next() {
                if character == '§' {
                    chars.next();
                } else {
                    out.push(character);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| flatten(item, out)),
        Value::Object(map) => {
            if let Some(text) = map.get("text") {
                flatten(text, out);
            }
            if let Some(extra) = map.get("extra") {
                flatten(extra, out);
            }
        }
        _ => {}
    }
}

fn status_from(json: &[u8], latency_ms: Option<u64>) -> AppResult<ServerStatus> {
    let value: Value = serde_json::from_slice(json)?;
    let mut motd = String::new();
    flatten(value.get("description").unwrap_or(&Value::Null), &mut motd);
    let number = |value: Option<&Value>| value.and_then(Value::as_u64).map_or(0, |count| count.min(u64::from(u32::MAX)) as u32);
    let players = value.get("players");
    let favicon = value
        .get("favicon")
        .and_then(Value::as_str)
        .filter(|icon| icon.starts_with("data:image/png;base64,") && icon.len() <= MAX_FAVICON)
        .map(str::to_owned);
    Ok(ServerStatus {
        motd: motd.trim().chars().take(MAX_MOTD).collect(),
        version: value.pointer("/version/name").and_then(Value::as_str).unwrap_or_default().chars().take(40).collect(),
        players_online: number(players.and_then(|players| players.get("online"))),
        players_max: number(players.and_then(|players| players.get("max"))),
        latency_ms,
        favicon,
    })
}

async fn read_status(stream: &mut TcpStream) -> AppResult<Vec<u8>> {
    let length = read_varint(stream).await?;
    if !(1..=MAX_RESPONSE as i32).contains(&length) {
        return Err(AppError::InvalidInput("the server sent an unexpected answer".into()));
    }
    let mut content = vec![0u8; length as usize];
    stream.read_exact(&mut content).await?;
    let unexpected = || AppError::InvalidInput("the server sent an unexpected answer".into());
    let mut cursor = 0;
    if take_varint(&content, &mut cursor)? != 0 {
        return Err(unexpected());
    }
    let size = usize::try_from(take_varint(&content, &mut cursor)?).map_err(|_| unexpected())?;
    let end = cursor.checked_add(size).ok_or_else(unexpected)?;
    content.get(cursor..end).map(<[u8]>::to_vec).ok_or_else(unexpected)
}

async fn ping_inner(host: &str, port: u16) -> AppResult<ServerStatus> {
    let mut stream = TcpStream::connect((host, port)).await?;
    stream.write_all(&handshake(host, port)).await?;
    stream.write_all(&packet(0, &[])).await?;
    let json = read_status(&mut stream).await?;

    let started = Instant::now();
    let latency_ms = match stream.write_all(&packet(1, &0i64.to_be_bytes())).await {
        Ok(()) => match read_varint(&mut stream).await {
            Ok(length) if (1..=16).contains(&length) => {
                let mut rest = vec![0u8; length as usize];
                stream.read_exact(&mut rest).await.ok().map(|_| started.elapsed().as_millis() as u64)
            }
            _ => None,
        },
        Err(_) => None,
    };
    status_from(&json, latency_ms)
}

/// Asks a server for its status the way the game's server list does.
pub async fn ping(address: &str) -> AppResult<ServerStatus> {
    let (host, port) = parse_address(address)?;
    timeout(TIMEOUT, ping_inner(&host, port))
        .await
        .map_err(|_| AppError::InvalidInput("the server did not answer in time".into()))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::net::TcpListener;

    fn scratch(name: &str) -> DataPaths {
        let root: PathBuf = std::env::temp_dir().join(format!("kairos-servers-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        DataPaths::new(root)
    }

    #[test]
    fn reads_hosts_with_and_without_a_port() {
        assert_eq!(parse_address("Play.Example.com").unwrap(), ("play.example.com".to_owned(), 25565));
        assert_eq!(parse_address(" 10.0.0.5:25570 ").unwrap(), ("10.0.0.5".to_owned(), 25570));
        for bad in ["", "host:0", "host:99999", "host:abc", "-bad.com", "bad host", "a/b", "http://x.com", "x.com:25565:1"] {
            assert!(parse_address(bad).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn keeps_a_list_of_servers() {
        let paths = scratch("list");
        let first = add(&paths, "  Survival ", "mc.example.com").unwrap();
        add(&paths, "Creative", "mc.example.com:25570").unwrap();
        assert_eq!(load(&paths).unwrap().iter().map(|server| server.address.as_str()).collect::<Vec<_>>(), ["mc.example.com", "mc.example.com:25570"]);
        assert_eq!(first.name, "Survival");

        remove(&paths, &first.id).unwrap();
        assert_eq!(load(&paths).unwrap().len(), 1);
        assert!(add(&paths, "", "x.com").is_err());
    }

    #[test]
    fn writes_and_reads_variable_length_numbers() {
        let mut buffer = Vec::new();
        write_varint(&mut buffer, 300);
        assert_eq!(buffer, [0xac, 0x02]);
        write_varint(&mut buffer, -1);
        assert_eq!(&buffer[2..], [0xff, 0xff, 0xff, 0xff, 0x0f]);
    }

    #[test]
    fn flattens_formatted_descriptions() {
        let mut text = String::new();
        flatten(&serde_json::json!({"text": "§aHello ", "extra": [{"text": "world"}, "!"]}), &mut text);
        assert_eq!(text, "Hello world!");
    }

    #[test]
    fn refuses_icons_that_are_not_png_data() {
        let status = status_from(br#"{"description":"A","favicon":"https://evil.example/x.png","players":{"online":3,"max":20},"version":{"name":"1.21"}}"#, Some(12)).unwrap();
        assert_eq!((status.players_online, status.players_max, status.latency_ms), (3, 20, Some(12)));
        assert!(status.favicon.is_none());
        let with_icon = status_from(br#"{"description":"A","favicon":"data:image/png;base64,AAAA"}"#, None).unwrap();
        assert!(with_icon.favicon.is_some());
    }

    #[tokio::test]
    async fn talks_to_a_server_through_the_status_exchange() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut seen = vec![0u8; 64];
            let _ = stream.read(&mut seen).await.unwrap();
            let json = br#"{"version":{"name":"Paper 1.21.1","protocol":767},"players":{"max":50,"online":7},"description":{"text":"Hello"}}"#;
            let mut content = Vec::new();
            write_varint(&mut content, 0);
            write_varint(&mut content, json.len() as i32);
            content.extend_from_slice(json);
            let mut framed = Vec::new();
            write_varint(&mut framed, content.len() as i32);
            framed.extend(content);
            stream.write_all(&framed).await.unwrap();

            let mut ping = vec![0u8; 16];
            let read = stream.read(&mut ping).await.unwrap();
            assert!(read > 0);
            stream.write_all(&packet(1, &0i64.to_be_bytes())).await.unwrap();
        });

        let status = ping(&format!("127.0.0.1:{port}")).await.unwrap();

        assert_eq!(status.motd, "Hello");
        assert_eq!(status.version, "Paper 1.21.1");
        assert_eq!((status.players_online, status.players_max), (7, 50));
        assert!(status.latency_ms.is_some());
    }

    #[tokio::test]
    async fn reports_an_error_when_nothing_listens() {
        let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        assert!(ping(&format!("127.0.0.1:{free}")).await.is_err());
    }
}
