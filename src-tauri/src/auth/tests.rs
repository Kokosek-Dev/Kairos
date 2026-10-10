use super::endpoints::Endpoints;
use super::store::memory::MemoryStore;
use super::store::{AccountStore, SecretStore};
use super::{AuthService, SignInEvent};
use crate::error::{AppError, AuthFailure};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

const CLIENT_ID: &str = "12345678-abcd-4ef0-9abc-0123456789ab";

type Responses = HashMap<&'static str, VecDeque<(u16, String)>>;

#[derive(Clone, Debug)]
struct Recorded {
    path: String,
    body: String,
    authorization: Option<String>,
}

struct Mock {
    base: String,
    requests: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    fn count(&self, path: &str) -> usize {
        self.requests.lock().unwrap().iter().filter(|request| request.path == path).count()
    }

    fn last(&self, path: &str) -> Recorded {
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

async fn start(routes: Vec<(&'static str, Vec<(u16, &str)>)>) -> Mock {
    let responses: Responses = routes
        .into_iter()
        .map(|(path, queue)| (path, queue.into_iter().map(|(status, body)| (status, body.to_owned())).collect()))
        .collect();
    let responses = Arc::new(Mutex::new(responses));
    let requests: Arc<Mutex<Vec<Recorded>>> = Arc::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());

    let log = requests.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else { return };
            let (responses, log) = (responses.clone(), log.clone());
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
                let path = head.split_whitespace().nth(1).unwrap_or_default().to_owned();
                let authorization = head
                    .lines()
                    .find(|line| line.to_lowercase().starts_with("authorization:"))
                    .map(|line| line.split_once(':').unwrap().1.trim().to_owned());
                let body = String::from_utf8_lossy(&data[head_end..]).into_owned();
                log.lock().unwrap().push(Recorded { path: path.clone(), body, authorization });

                let (status, body) = {
                    let mut responses = responses.lock().unwrap();
                    match responses.get_mut(path.as_str()) {
                        Some(queue) if queue.len() > 1 => queue.pop_front().unwrap(),
                        Some(queue) => queue.front().cloned().unwrap_or((404, "{}".into())),
                        None => (404, "{}".into()),
                    }
                };
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

const DEVICE: &str = r#"{"device_code":"DC","user_code":"ABCD-1234","verification_uri":"https://www.microsoft.com/link","expires_in":900,"interval":0}"#;
const XBOX: &str = r#"{"Token":"XBOX-TOKEN","DisplayClaims":{"xui":[{"uhs":"USERHASH"}]}}"#;
const XSTS: &str = r#"{"Token":"XSTS-TOKEN","DisplayClaims":{"xui":[{"uhs":"USERHASH"}]}}"#;
const MC_LOGIN: &str = r#"{"access_token":"MC-TOKEN","expires_in":86400}"#;
const PROFILE: &str = r#"{"id":"0123456789abcdef0123456789abcdef","name":"Steve"}"#;

fn happy_routes() -> Vec<(&'static str, Vec<(u16, &'static str)>)> {
    vec![
        ("/devicecode", vec![(200, DEVICE)]),
        (
            "/token",
            vec![
                (400, r#"{"error":"authorization_pending"}"#),
                (200, r#"{"access_token":"MS-ACCESS","refresh_token":"REFRESH-1"}"#),
            ],
        ),
        ("/xbl", vec![(200, XBOX)]),
        ("/xsts", vec![(200, XSTS)]),
        ("/mc/login", vec![(200, MC_LOGIN)]),
        ("/mc/profile", vec![(200, PROFILE)]),
    ]
}

struct Fixture {
    service: AuthService,
    secrets: Arc<MemoryStore>,
    mock: Mock,
}

async fn fixture(routes: Vec<(&'static str, Vec<(u16, &'static str)>)>) -> Fixture {
    let mock = start(routes).await;
    let secrets = Arc::new(MemoryStore::default());
    let file = std::env::temp_dir().join(format!(
        "zenith-auth-{}-{}.json",
        std::process::id(),
        mock.base.rsplit(':').next().unwrap()
    ));
    let _ = std::fs::remove_file(&file);
    let store = AccountStore::new(file, secrets.clone() as Arc<dyn SecretStore>);
    let service = AuthService::new(reqwest::Client::new(), Endpoints::with_base(&mock.base), store);
    Fixture { service, secrets, mock }
}

async fn sign_in(fixture: &Fixture) -> Result<(super::store::Account, Vec<SignInEvent>), AppError> {
    let events: Mutex<Vec<SignInEvent>> = Mutex::default();
    let account = fixture
        .service
        .sign_in(CLIENT_ID, &CancellationToken::new(), |event| events.lock().unwrap().push(event))
        .await?;
    Ok((account, events.into_inner().unwrap()))
}

#[tokio::test]
async fn signs_in_through_every_step_and_keeps_only_the_refresh_token() {
    let fixture = fixture(happy_routes()).await;

    let (account, events) = sign_in(&fixture).await.unwrap();

    assert_eq!((account.username.as_str(), account.id.as_str()), ("Steve", "0123456789abcdef0123456789abcdef"));
    assert!(matches!(&events[0], SignInEvent::DeviceCode { user_code, .. } if user_code == "ABCD-1234"));
    let steps: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            SignInEvent::Step { step } => Some(*step),
            _ => None,
        })
        .map(|step| format!("{step:?}"))
        .collect();
    assert_eq!(steps, ["Xbox", "Xsts", "Minecraft", "Profile"]);

    assert!(fixture.mock.last("/xbl").body.contains("d=MS-ACCESS"));
    assert!(fixture.mock.last("/xsts").body.contains("rp://api.minecraftservices.com/"));
    assert!(fixture.mock.last("/mc/login").body.contains("XBL3.0 x=USERHASH;XSTS-TOKEN"));
    assert_eq!(fixture.mock.last("/mc/profile").authorization.as_deref(), Some("Bearer MC-TOKEN"));
    assert!(fixture.mock.last("/devicecode").body.contains("XboxLive.signin"));

    let stored = fixture.secrets.get(&account.id).unwrap().unwrap();
    assert_eq!(stored, b"REFRESH-1");
    let accounts = fixture.service.accounts().await.unwrap();
    assert_eq!(accounts.active_id.as_deref(), Some(account.id.as_str()));
}

#[tokio::test]
async fn reuses_the_cached_minecraft_token_for_the_next_launch() {
    let fixture = fixture(happy_routes()).await;
    sign_in(&fixture).await.unwrap();
    let token_requests = fixture.mock.count("/token");

    let session = fixture.service.game_session(|| Ok(CLIENT_ID.to_owned())).await.unwrap();

    assert_eq!(session.access_token, "MC-TOKEN");
    assert_eq!(session.username, "Steve");
    assert_eq!(session.user_type, "msa");
    assert_eq!(fixture.mock.count("/token"), token_requests);
}

#[tokio::test]
async fn refreshes_the_session_and_rotates_the_stored_refresh_token() {
    let mut routes = happy_routes();
    routes[1] = (
        "/token",
        vec![
            (400, r#"{"error":"authorization_pending"}"#),
            (200, r#"{"access_token":"MS-ACCESS","refresh_token":"REFRESH-1"}"#),
            (200, r#"{"access_token":"MS-ACCESS-2","refresh_token":"REFRESH-2"}"#),
        ],
    );
    let fixture = fixture(routes).await;
    let (account, _) = sign_in(&fixture).await.unwrap();
    fixture.service.cache.lock().await.clear();

    let session = fixture.service.game_session(|| Ok(CLIENT_ID.to_owned())).await.unwrap();

    assert_eq!(session.access_token, "MC-TOKEN");
    assert!(fixture.mock.last("/token").body.contains("refresh_token=REFRESH-1"));
    assert!(fixture.mock.last("/xbl").body.contains("d=MS-ACCESS-2"));
    assert_eq!(fixture.secrets.get(&account.id).unwrap().unwrap(), b"REFRESH-2");
}

#[tokio::test]
async fn asks_to_sign_in_again_when_the_refresh_token_is_no_longer_valid() {
    let mut routes = happy_routes();
    routes[1] = (
        "/token",
        vec![
            (200, r#"{"access_token":"MS-ACCESS","refresh_token":"REFRESH-1"}"#),
            (400, r#"{"error":"invalid_grant"}"#),
        ],
    );
    let fixture = fixture(routes).await;
    sign_in(&fixture).await.unwrap();
    fixture.service.cache.lock().await.clear();

    let result = fixture.service.game_session(|| Ok(CLIENT_ID.to_owned())).await;

    assert!(matches!(result, Err(AppError::Auth(AuthFailure::SessionExpired))));
}

#[tokio::test]
async fn refuses_to_launch_without_an_account() {
    let fixture = fixture(happy_routes()).await;

    let result = fixture.service.game_session(|| Ok(CLIENT_ID.to_owned())).await;

    assert!(matches!(result, Err(AppError::NotSignedIn)));
}

#[tokio::test]
async fn reports_accounts_without_an_xbox_profile() {
    let mut routes = happy_routes();
    routes[3] = ("/xsts", vec![(401, r#"{"Identity":"0","XErr":2148916233,"Message":"","Redirect":"x"}"#)]);
    let fixture = fixture(routes).await;

    let result = sign_in(&fixture).await;

    assert!(matches!(result, Err(AppError::Auth(AuthFailure::XboxNoAccount))));
    assert!(fixture.service.accounts().await.unwrap().accounts.is_empty());
}

#[tokio::test]
async fn reports_child_accounts() {
    let mut routes = happy_routes();
    routes[3] = ("/xsts", vec![(401, r#"{"XErr":2148916238}"#)]);
    let fixture = fixture(routes).await;

    assert!(matches!(sign_in(&fixture).await, Err(AppError::Auth(AuthFailure::XboxChildAccount))));
}

#[tokio::test]
async fn explains_that_an_unapproved_application_is_rejected_by_minecraft_services() {
    let mut routes = happy_routes();
    routes[4] = ("/mc/login", vec![(403, r#"{"error":"Invalid app registration"}"#)]);
    let fixture = fixture(routes).await;

    assert!(matches!(sign_in(&fixture).await, Err(AppError::Auth(AuthFailure::AppNotApproved))));
}

#[tokio::test]
async fn tells_accounts_without_the_game_apart_from_missing_profiles() {
    let mut routes = happy_routes();
    routes[5] = ("/mc/profile", vec![(404, r#"{"error":"NOT_FOUND"}"#)]);
    routes.push(("/mc/entitlements", vec![(200, r#"{"items":[]}"#)]));
    let fixture = fixture(routes.clone()).await;
    assert!(matches!(sign_in(&fixture).await, Err(AppError::Auth(AuthFailure::GameNotOwned))));

    routes.pop();
    routes.push(("/mc/entitlements", vec![(200, r#"{"items":[{"name":"game_minecraft"}]}"#)]));
    let fixture = self::fixture(routes).await;
    assert!(matches!(sign_in(&fixture).await, Err(AppError::Auth(AuthFailure::ProfileMissing))));
}

#[tokio::test]
async fn stops_when_the_user_declines_or_the_code_expires() {
    let mut routes = happy_routes();
    routes[1] = ("/token", vec![(400, r#"{"error":"authorization_declined"}"#)]);
    let declined = fixture(routes.clone()).await;
    assert!(matches!(sign_in(&declined).await, Err(AppError::Auth(AuthFailure::Declined))));

    routes[1] = ("/token", vec![(400, r#"{"error":"expired_token"}"#)]);
    let expired = fixture(routes).await;
    assert!(matches!(sign_in(&expired).await, Err(AppError::Auth(AuthFailure::Expired))));
}

#[tokio::test]
async fn sign_in_can_be_cancelled_while_waiting_for_the_user() {
    let mut routes = happy_routes();
    routes[1] = ("/token", vec![(400, r#"{"error":"authorization_pending"}"#)]);
    let fixture = fixture(routes).await;
    let cancel = CancellationToken::new();
    cancel.cancel();

    let result = fixture.service.sign_in(CLIENT_ID, &cancel, |_| {}).await;

    assert!(matches!(result, Err(AppError::Cancelled)));
}

#[tokio::test]
async fn removing_an_account_deletes_its_secret_and_promotes_another() {
    let fixture = fixture(happy_routes()).await;
    let (account, _) = sign_in(&fixture).await.unwrap();

    fixture.service.remove(account.id.clone()).await.unwrap();

    assert!(fixture.secrets.get(&account.id).unwrap().is_none());
    assert!(fixture.service.accounts().await.unwrap().accounts.is_empty());
}
