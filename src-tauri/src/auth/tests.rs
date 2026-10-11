use super::endpoints::Endpoints;
use super::store::memory::MemoryStore;
use super::store::{AccountStore, SecretStore};
use super::{AuthService, SignInEvent};
use crate::error::{AppError, AuthFailure};
use crate::test_support::{start, Mock};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

const CLIENT_ID: &str = "12345678-abcd-4ef0-9abc-0123456789ab";
const XBOX: &str = r#"{"Token":"XBOX-TOKEN","DisplayClaims":{"xui":[{"uhs":"USERHASH"}]}}"#;
const XSTS: &str = r#"{"Token":"XSTS-TOKEN","DisplayClaims":{"xui":[{"uhs":"USERHASH"}]}}"#;
const MC_LOGIN: &str = r#"{"access_token":"MC-TOKEN","expires_in":86400}"#;
const PROFILE: &str = r#"{"id":"0123456789abcdef0123456789abcdef","name":"Steve"}"#;
const TOKENS: &str = r#"{"access_token":"MS-ACCESS","refresh_token":"REFRESH-1"}"#;

fn happy_routes() -> Vec<(&'static str, Vec<(u16, &'static str)>)> {
    vec![
        ("/token", vec![(200, TOKENS)]),
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
        "kairos-auth-{}-{}.json",
        std::process::id(),
        mock.base.rsplit(':').next().unwrap()
    ));
    let _ = std::fs::remove_file(&file);
    let store = AccountStore::new(file, secrets.clone() as Arc<dyn SecretStore>);
    let service = AuthService::new(reqwest::Client::new(), Endpoints::with_base(&mock.base), store);
    Fixture { service, secrets, mock }
}

#[derive(Clone, Copy)]
enum Browser {
    Approves,
    Denies,
    NeverReturns,
}

type SignInResult = Result<(super::store::Account, Vec<SignInEvent>, Option<String>), AppError>;

/// Plays the part of the user's browser: reads the authorize URL and, like Microsoft would,
/// redirects back to the launcher's loopback address.
async fn sign_in_with(fixture: &Fixture, browser: Browser, cancel: &CancellationToken) -> SignInResult {
    let events: Mutex<Vec<SignInEvent>> = Mutex::default();
    let authorize_url: Arc<Mutex<Option<String>>> = Arc::default();
    let seen = authorize_url.clone();

    let open_browser = move |url: &str| {
        *seen.lock().unwrap() = Some(url.to_owned());
        let parsed = reqwest::Url::parse(url).unwrap();
        let param = |name: &str| parsed.query_pairs().find(|(key, _)| key == name).unwrap().1.into_owned();
        let (redirect, state) = (param("redirect_uri"), param("state"));
        let query = match browser {
            Browser::Approves => format!("code=AUTH-CODE&state={state}"),
            Browser::Denies => format!("error=access_denied&state={state}"),
            Browser::NeverReturns => return Ok(()),
        };
        tokio::spawn(async move {
            let _ = reqwest::get(format!("{redirect}/?{query}")).await;
        });
        Ok(())
    };

    let account = fixture
        .service
        .sign_in(CLIENT_ID, cancel, open_browser, |event| events.lock().unwrap().push(event))
        .await?;
    let url = authorize_url.lock().unwrap().clone();
    Ok((account, events.into_inner().unwrap(), url))
}

async fn sign_in(fixture: &Fixture) -> SignInResult {
    sign_in_with(fixture, Browser::Approves, &CancellationToken::new()).await
}

fn form_value(body: &str, name: &str) -> String {
    reqwest::Url::parse(&format!("http://x/?{body}"))
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == name)
        .unwrap_or_else(|| panic!("missing {name} in {body}"))
        .1
        .into_owned()
}

#[tokio::test]
async fn signs_in_through_the_browser_and_every_step_keeping_only_the_refresh_token() {
    let fixture = fixture(happy_routes()).await;

    let (account, events, url) = sign_in(&fixture).await.unwrap();

    assert_eq!((account.username.as_str(), account.id.as_str()), ("Steve", "0123456789abcdef0123456789abcdef"));
    assert!(matches!(&events[0], SignInEvent::Browser { .. }));
    let steps: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            SignInEvent::Step { step } => Some(format!("{step:?}")),
            _ => None,
        })
        .collect();
    assert_eq!(steps, ["Xbox", "Xsts", "Minecraft", "Profile"]);

    let authorize = reqwest::Url::parse(&url.unwrap()).unwrap();
    let query = authorize.query().unwrap();
    assert_eq!(form_value(query, "client_id"), CLIENT_ID);
    assert_eq!(form_value(query, "response_type"), "code");
    assert!(form_value(query, "scope").contains("XboxLive.signin"));
    assert!(form_value(query, "redirect_uri").starts_with("http://localhost:"));
    assert_eq!(form_value(query, "code_challenge_method"), "S256");

    let exchange = fixture.mock.last("/token").body;
    assert_eq!(form_value(&exchange, "grant_type"), "authorization_code");
    assert_eq!(form_value(&exchange, "code"), "AUTH-CODE");
    assert_eq!(form_value(&exchange, "redirect_uri"), form_value(query, "redirect_uri"));
    let verifier = form_value(&exchange, "code_verifier");
    let expected_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    assert_eq!(form_value(query, "code_challenge"), expected_challenge);
    assert!(!exchange.contains("client_secret"));

    assert!(fixture.mock.last("/xbl").body.contains("d=MS-ACCESS"));
    assert!(fixture.mock.last("/xsts").body.contains("rp://api.minecraftservices.com/"));
    assert!(fixture.mock.last("/mc/login").body.contains("XBL3.0 x=USERHASH;XSTS-TOKEN"));
    assert_eq!(fixture.mock.last("/mc/profile").authorization.as_deref(), Some("Bearer MC-TOKEN"));

    assert_eq!(fixture.secrets.get(&account.id).unwrap().unwrap(), b"REFRESH-1");
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
    routes[0] = (
        "/token",
        vec![
            (200, TOKENS),
            (200, r#"{"access_token":"MS-ACCESS-2","refresh_token":"REFRESH-2"}"#),
        ],
    );
    let fixture = fixture(routes).await;
    let (account, _, _) = sign_in(&fixture).await.unwrap();
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
    routes[0] = ("/token", vec![(200, TOKENS), (400, r#"{"error":"invalid_grant"}"#)]);
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
    routes[2] = ("/xsts", vec![(401, r#"{"Identity":"0","XErr":2148916233,"Message":"","Redirect":"x"}"#)]);
    let fixture = fixture(routes).await;

    let result = sign_in(&fixture).await;

    assert!(matches!(result, Err(AppError::Auth(AuthFailure::XboxNoAccount))));
    assert!(fixture.service.accounts().await.unwrap().accounts.is_empty());
}

#[tokio::test]
async fn reports_child_accounts() {
    let mut routes = happy_routes();
    routes[2] = ("/xsts", vec![(401, r#"{"XErr":2148916238}"#)]);
    let fixture = fixture(routes).await;

    assert!(matches!(sign_in(&fixture).await, Err(AppError::Auth(AuthFailure::XboxChildAccount))));
}

#[tokio::test]
async fn explains_that_an_unapproved_application_is_rejected_by_minecraft_services() {
    let mut routes = happy_routes();
    routes[3] = ("/mc/login", vec![(403, r#"{"error":"Invalid app registration"}"#)]);
    let fixture = fixture(routes).await;

    assert!(matches!(sign_in(&fixture).await, Err(AppError::Auth(AuthFailure::AppNotApproved))));
}

#[tokio::test]
async fn tells_accounts_without_the_game_apart_from_missing_profiles() {
    let mut routes = happy_routes();
    routes[4] = ("/mc/profile", vec![(404, r#"{"error":"NOT_FOUND"}"#)]);
    routes.push(("/mc/entitlements", vec![(200, r#"{"items":[]}"#)]));
    let fixture = fixture(routes.clone()).await;
    assert!(matches!(sign_in(&fixture).await, Err(AppError::Auth(AuthFailure::GameNotOwned))));

    routes.pop();
    routes.push(("/mc/entitlements", vec![(200, r#"{"items":[{"name":"game_minecraft"}]}"#)]));
    let fixture = self::fixture(routes).await;
    assert!(matches!(sign_in(&fixture).await, Err(AppError::Auth(AuthFailure::ProfileMissing))));
}

#[tokio::test]
async fn stops_when_the_user_declines_in_the_browser() {
    let fixture = fixture(happy_routes()).await;

    let result = sign_in_with(&fixture, Browser::Denies, &CancellationToken::new()).await;

    assert!(matches!(result, Err(AppError::Auth(AuthFailure::Declined))));
    assert_eq!(fixture.mock.count("/token"), 0);
}

#[tokio::test]
async fn sign_in_can_be_cancelled_while_waiting_for_the_browser() {
    let fixture = fixture(happy_routes()).await;
    let cancel = CancellationToken::new();
    let trigger = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        trigger.cancel();
    });

    let result = sign_in_with(&fixture, Browser::NeverReturns, &cancel).await;

    assert!(matches!(result, Err(AppError::Cancelled)));
}

#[tokio::test]
async fn removing_an_account_deletes_its_secret_and_promotes_another() {
    let fixture = fixture(happy_routes()).await;
    let (account, _, _) = sign_in(&fixture).await.unwrap();

    fixture.service.remove(account.id.clone()).await.unwrap();

    assert!(fixture.secrets.get(&account.id).unwrap().is_none());
    assert!(fixture.service.accounts().await.unwrap().accounts.is_empty());
}

#[tokio::test]
async fn offline_play_uses_the_saved_account_without_touching_the_network() {
    let fixture = fixture(happy_routes()).await;
    let (account, _, _) = sign_in(&fixture).await.unwrap();
    let requests_after_sign_in = fixture.mock.requests.lock().unwrap().len();

    let session = fixture.service.offline_session().await.unwrap();

    assert_eq!((session.username.as_str(), session.uuid.as_str()), ("Steve", account.id.as_str()));
    assert_ne!(session.access_token, "MC-TOKEN");
    assert_eq!(fixture.mock.requests.lock().unwrap().len(), requests_after_sign_in);
}

#[tokio::test]
async fn offline_play_is_refused_without_a_verified_account() {
    let fixture = fixture(happy_routes()).await;

    assert!(matches!(fixture.service.offline_session().await, Err(AppError::NotSignedIn)));
}
