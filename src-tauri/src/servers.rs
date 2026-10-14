use crate::error::{AppError, AppResult};
use crate::paths::DataPaths;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;

const JAVA_PORT: u16 = 25565;
const BEDROCK_PORT: u16 = 19132;
/// The 16 bytes that mark every RakNet "offline" message, the same in every Bedrock server list query.
const RAKNET_MAGIC: [u8; 16] = [0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78];
const UNCONNECTED_PING: u8 = 0x01;
const UNCONNECTED_PONG: u8 = 0x1c;
const MAX_BEDROCK_REPLY: usize = 2048;
const TIMEOUT: Duration = Duration::from_secs(5);
const MAX_SERVERS: usize = 100;
const MAX_NAME: usize = 48;
const MAX_ADDRESS: usize = 253;
const MAX_RESPONSE: usize = 512 * 1024;
const MAX_FAVICON: usize = 100 * 1024;
const MAX_MOTD: usize = 200;
/// The protocol number sent in the handshake. Servers answer a status request whatever it is.
const HANDSHAKE_PROTOCOL: i32 = 765;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Edition {
    #[default]
    Java,
    Bedrock,
}

impl Edition {
    fn default_port(self) -> u16 {
        match self {
            Self::Java => JAVA_PORT,
            Self::Bedrock => BEDROCK_PORT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerEntry {
    pub id: String,
    pub name: String,
    pub address: String,
    /// Lists saved before Bedrock was added have no edition and are Java servers.
    #[serde(default)]
    pub edition: Edition,
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

/// Splits `host` or `host:port`, using the edition's usual port when none is given.
/// Anything that is not a plain host name or address is refused.
pub fn parse_address(address: &str, edition: Edition) -> AppResult<(String, u16)> {
    let invalid = || AppError::InvalidInput("invalid server address".into());
    let address = address.trim();
    if address.is_empty() || address.len() > MAX_ADDRESS {
        return Err(invalid());
    }
    let (host, port) = match address.rsplit_once(':') {
        Some((host, port)) => (host, port.parse::<u16>().ok().filter(|port| *port > 0).ok_or_else(invalid)?),
        None => (address, edition.default_port()),
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

pub fn add(paths: &DataPaths, name: &str, address: &str, edition: Edition) -> AppResult<ServerEntry> {
    let (host, port) = parse_address(address, edition)?;
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
        address: if port == edition.default_port() { host } else { format!("{host}:{port}") },
        edition,
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

/// Bedrock Edition installs from the Microsoft Store into a per-user package folder.
pub fn bedrock_installed() -> bool {
    std::env::var_os("LOCALAPPDATA").is_some_and(|local| std::path::Path::new(&local).join("Packages").join("Microsoft.MinecraftUWP_8wekyb3d8bbwe").is_dir())
}
/// The request a Bedrock game sends to fill its server list (a RakNet unconnected ping).
fn bedrock_query(sent_at_ms: u64) -> Vec<u8> {
    let mut query = vec![UNCONNECTED_PING];
    query.extend_from_slice(&sent_at_ms.to_be_bytes());
    query.extend_from_slice(&RAKNET_MAGIC);
    query.extend_from_slice(&0u64.to_be_bytes());
    query
}

/// Reads the answer, whose text is `MCPE;name;protocol;version;online;max;...`.
fn bedrock_status(reply: &[u8], latency_ms: Option<u64>) -> AppResult<ServerStatus> {
    let unexpected = || AppError::InvalidInput("the server sent an unexpected answer".into());
    // The answer is: type, time (8), server id (8), magic (16), text length (2), text.
    if reply.len() < 35 || reply[0] != UNCONNECTED_PONG || reply[17..33] != RAKNET_MAGIC {
        return Err(unexpected());
    }
    let length = usize::from(u16::from_be_bytes([reply[33], reply[34]]));
    let text = String::from_utf8_lossy(reply.get(35..35 + length).ok_or_else(unexpected)?).into_owned();
    let fields: Vec<&str> = text.split(';').collect();
    if fields.len() < 6 {
        return Err(unexpected());
    }
    let number = |index: usize| fields[index].trim().parse::<u32>().unwrap_or(0);
    let mut motd = String::new();
    flatten(&Value::String(fields[1].to_owned()), &mut motd);
    Ok(ServerStatus {
        motd: motd.trim().chars().take(MAX_MOTD).collect(),
        version: fields[3].chars().take(40).collect(),
        players_online: number(4),
        players_max: number(5),
        latency_ms,
        favicon: None,
    })
}

async fn ping_bedrock(host: &str, port: u16) -> AppResult<ServerStatus> {
    let target = tokio::net::lookup_host((host, port))
        .await?
        .find(std::net::SocketAddr::is_ipv4)
        .ok_or_else(|| AppError::InvalidInput("could not find that server".into()))?;
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(target).await?;
    let started = Instant::now();
    socket.send(&bedrock_query(chrono::Utc::now().timestamp_millis().unsigned_abs())).await?;
    let mut buffer = vec![0u8; MAX_BEDROCK_REPLY];
    let read = socket.recv(&mut buffer).await?;
    bedrock_status(&buffer[..read], Some(started.elapsed().as_millis() as u64))
}

/// Asks a server for its status the way the game's server list does.
pub async fn ping(address: &str, edition: Edition) -> AppResult<ServerStatus> {
    let (host, port) = parse_address(address, edition)?;
    let asked = async {
        match edition {
            Edition::Java => ping_inner(&host, port).await,
            Edition::Bedrock => ping_bedrock(&host, port).await,
        }
    };
    timeout(TIMEOUT, asked)
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
        assert_eq!(parse_address("Play.Example.com", Edition::Java).unwrap(), ("play.example.com".to_owned(), 25565));
        assert_eq!(parse_address(" 10.0.0.5:25570 ", Edition::Java).unwrap(), ("10.0.0.5".to_owned(), 25570));
        for bad in ["", "host:0", "host:99999", "host:abc", "-bad.com", "bad host", "a/b", "http://x.com", "x.com:25565:1"] {
            assert!(parse_address(bad, Edition::Java).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn keeps_a_list_of_servers() {
        let paths = scratch("list");
        let first = add(&paths, "  Survival ", "mc.example.com", Edition::Java).unwrap();
        add(&paths, "Creative", "mc.example.com:25570", Edition::Java).unwrap();
        assert_eq!(load(&paths).unwrap().iter().map(|server| server.address.as_str()).collect::<Vec<_>>(), ["mc.example.com", "mc.example.com:25570"]);
        assert_eq!(first.name, "Survival");

        remove(&paths, &first.id).unwrap();
        assert_eq!(load(&paths).unwrap().len(), 1);
        assert!(add(&paths, "", "x.com", Edition::Java).is_err());
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

        let status = ping(&format!("127.0.0.1:{port}"), Edition::Java).await.unwrap();

        assert_eq!(status.motd, "Hello");
        assert_eq!(status.version, "Paper 1.21.1");
        assert_eq!((status.players_online, status.players_max), (7, 50));
        assert!(status.latency_ms.is_some());
    }

    #[tokio::test]
    async fn reports_an_error_when_nothing_listens() {
        let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        assert!(ping(&format!("127.0.0.1:{free}"), Edition::Java).await.is_err());
    }

    fn pong(text: &str) -> Vec<u8> {
        let mut reply = vec![UNCONNECTED_PONG];
        reply.extend_from_slice(&1234u64.to_be_bytes());
        reply.extend_from_slice(&99u64.to_be_bytes());
        reply.extend_from_slice(&RAKNET_MAGIC);
        reply.extend_from_slice(&(text.len() as u16).to_be_bytes());
        reply.extend_from_slice(text.as_bytes());
        reply
    }

    #[test]
    fn reads_what_a_bedrock_server_answers_with() {
        // Taken from a real answer of a public server.
        let status = bedrock_status(&pong("MCPE;§f§lHIVE-O-WEEN | BEDWARS S5;121;1.0;12136;100001;2561061765108221877;Hive Games;Survival"), Some(40)).unwrap();

        assert_eq!(status.motd, "HIVE-O-WEEN | BEDWARS S5");
        assert_eq!(status.version, "1.0");
        assert_eq!((status.players_online, status.players_max, status.latency_ms), (12136, 100001, Some(40)));
        assert!(status.favicon.is_none());
    }

    #[test]
    fn refuses_answers_that_are_not_a_bedrock_status() {
        assert!(bedrock_status(&[], None).is_err());
        assert!(bedrock_status(&[UNCONNECTED_PONG; 10], None).is_err());
        let mut wrong_magic = pong("MCPE;A;1;1.0;1;2");
        wrong_magic[20] ^= 0xff;
        assert!(bedrock_status(&wrong_magic, None).is_err());
        let mut too_long = pong("MCPE;A;1;1.0;1;2");
        too_long[33] = 0xff;
        assert!(bedrock_status(&too_long, None).is_err());
        assert!(bedrock_status(&pong("MCPE;only;three"), None).is_err());
    }

    #[test]
    fn the_bedrock_query_has_the_shape_servers_expect() {
        let query = bedrock_query(0x0102_0304_0506_0708);

        assert_eq!(query.len(), 33);
        assert_eq!(query[0], UNCONNECTED_PING);
        assert_eq!(&query[1..9], [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(&query[9..25], RAKNET_MAGIC);
    }

    #[tokio::test]
    async fn asks_a_bedrock_server_over_udp() {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = socket.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let mut buffer = [0u8; 64];
            let (read, from) = socket.recv_from(&mut buffer).await.unwrap();
            assert_eq!(read, 33);
            assert_eq!(buffer[0], UNCONNECTED_PING);
            socket.send_to(&pong("MCPE;My world;800;1.26.50;3;10;77;Second line;Survival"), from).await.unwrap();
        });

        let status = ping(&format!("127.0.0.1:{port}"), Edition::Bedrock).await.unwrap();

        server.await.unwrap();
        assert_eq!((status.motd.as_str(), status.version.as_str()), ("My world", "1.26.50"));
        assert_eq!((status.players_online, status.players_max), (3, 10));
        assert!(status.latency_ms.is_some());
    }

    #[test]
    fn lists_saved_before_bedrock_existed_are_java_servers() {
        let entries: Vec<ServerEntry> = serde_json::from_str(r#"[{"id":"a","name":"Old","address":"mc.example.com"}]"#).unwrap();
        assert_eq!(entries[0].edition, Edition::Java);
    }

    #[test]
    fn keeps_the_edition_and_its_usual_port() {
        let paths = scratch("edition");
        let bedrock = add(&paths, "Hive", "geo.hivebedrock.network", Edition::Bedrock).unwrap();
        add(&paths, "Other port", "geo.hivebedrock.network:19133", Edition::Bedrock).unwrap();

        assert_eq!(bedrock.address, "geo.hivebedrock.network");
        let saved = load(&paths).unwrap();
        assert!(saved.iter().all(|server| server.edition == Edition::Bedrock));
        assert_eq!(saved[1].address, "geo.hivebedrock.network:19133");
    }
}