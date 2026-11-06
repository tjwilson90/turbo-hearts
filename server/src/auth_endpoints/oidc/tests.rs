use super::*;
use crate::Database;
use openidconnect::{
    core::{CoreIdToken, CoreIdTokenClaims, CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey},
    JsonWebKeyId, PrivateSigningKey,
};
use serde_json::{json, Value};
use std::{
    sync::Mutex as StdMutex,
    time::{SystemTime, UNIX_EPOCH},
};

// Public test fixtures only; these keys never authenticate a real provider.
fn signing_key(other: bool) -> CoreRsaPrivateSigningKey {
    CoreRsaPrivateSigningKey::from_pem(
        if other {
            include_str!("test-other-key.pem")
        } else {
            include_str!("test-key.pem")
        },
        Some(JsonWebKeyId::new("test-key".into())),
    )
    .unwrap()
}

struct MockState {
    token: String,
    userinfo: Value,
    key: Value,
    forms: Vec<HashMap<String, String>>,
    authorization: Vec<Option<String>>,
    wrong_discovery_issuer: bool,
}
struct MockProvider {
    issuer: String,
    state: Arc<StdMutex<MockState>>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for MockProvider {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl MockProvider {
    async fn new() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let issuer = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(StdMutex::new(MockState {
            token: String::new(),
            userinfo: json!({"sub":"alice", "name":"UserInfo Alice"}),
            key: serde_json::to_value(signing_key(false).as_verification_key()).unwrap(),
            forms: Vec::new(),
            authorization: Vec::new(),
            wrong_discovery_issuer: false,
        }));
        let data = Arc::clone(&state);
        let base = issuer.clone();
        let discovery = warp::path!(".well-known" / "openid-configuration").map(move || {
            let bad = data.lock().unwrap().wrong_discovery_issuer;
            warp::reply::json(&json!({
                "issuer": if bad { "https://wrong.example" } else { &base },
                "authorization_endpoint": format!("{base}/authorize"),
                "token_endpoint": format!("{base}/token"),
                "userinfo_endpoint": format!("{base}/userinfo"),
                "jwks_uri": format!("{base}/jwks"),
                "response_types_supported": ["code"], "subject_types_supported": ["public"],
                "id_token_signing_alg_values_supported": ["RS256"],
                "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "none"],
                "code_challenge_methods_supported": ["S256"]
            }))
        });
        let data = Arc::clone(&state);
        let jwks = warp::path("jwks")
            .map(move || warp::reply::json(&json!({"keys":[data.lock().unwrap().key.clone()]})));
        let data = Arc::clone(&state);
        let token = warp::path("token").and(warp::post()).and(warp::body::form())
            .and(warp::header::optional::<String>("authorization"))
            .map(move |form: HashMap<String, String>, auth: Option<String>| {
                let mut data = data.lock().unwrap(); data.forms.push(form); data.authorization.push(auth);
                warp::reply::json(&json!({"access_token":"access-token", "token_type":"Bearer", "id_token":data.token}))
            });
        let data = Arc::clone(&state);
        let info =
            warp::path("userinfo").map(move || warp::reply::json(&data.lock().unwrap().userinfo));
        let task = tokio::spawn(
            warp::serve(discovery.or(jwks).or(token).or(info))
                .incoming(listener)
                .run(),
        );
        Self {
            issuer,
            state,
            task,
        }
    }
    fn configure(&self, oidc_name: &str) -> BTreeMap<String, ProviderConfig> {
        BTreeMap::from([(
            oidc_name.into(),
            ProviderConfig {
                display_name: "Example & <Friends>".into(),
                issuer: self.issuer.clone(),
                client_id: "test-client".into(),
                client_secret: Some("test-secret".into()),
                client_secret_env: None,
                scopes: default_scopes(),
                client_auth_method: ClientAuthMethod::Basic,
            },
        )])
    }
    fn token(&self, nonce: &str, other_key: bool, change: impl FnOnce(&mut Value)) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut claims = json!({"iss":self.issuer, "sub":"alice", "aud":"test-client", "exp":now+600, "iat":now,
            "nonce":nonce, "name":"Alice; ❤️\r\nTest"});
        change(&mut claims);
        let claims: CoreIdTokenClaims = serde_json::from_value(claims).unwrap();
        let access_token = openidconnect::AccessToken::new("access-token".into());
        let hash_token = if claims.access_token_hash().is_none() {
            Some(&access_token)
        } else {
            None
        };
        let token = CoreIdToken::new(
            claims,
            &signing_key(other_key),
            CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
            hash_token,
            None,
        )
        .unwrap();
        self.state.lock().unwrap().token = serde_json::to_value(token)
            .unwrap()
            .as_str()
            .unwrap()
            .into();
    }
}

struct App {
    _dir: tempfile::TempDir,
    db: &'static Database,
    users: &'static Users,
    oidc: Arc<Oidc>,
}
impl App {
    fn new(config: BTreeMap<String, ProviderConfig>) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = &*Box::leak(Box::new(Database::new(dir.path().join("auth.db")).unwrap()));
        let users = &*Box::leak(Box::new(Users::new(db)));
        let oidc = Arc::new(Oidc::new(&config, "https://game.example").unwrap());
        Self {
            _dir: dir,
            db,
            users,
            oidc,
        }
    }
    fn routes(
        &self,
    ) -> impl Filter<Extract = (impl Reply,), Error = std::convert::Infallible> + Clone + use<>
    {
        let users = self.users;
        router(warp::any().map(move || users), Arc::clone(&self.oidc))
            .recover(crate::error::handle_rejection)
    }
    async fn begin(&self) -> (HashMap<String, String>, String) {
        let response = warp::test::request()
            .path("/auth/oidc/example")
            .reply(&self.routes())
            .await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let url = Url::parse(response.headers()[header::LOCATION].to_str().unwrap()).unwrap();
        let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params["response_type"], "code");
        assert_eq!(params["code_challenge_method"], "S256");
        assert_eq!(params["scope"], "openid profile");
        assert_eq!(
            params["redirect_uri"],
            "https://game.example/auth/oidc/example/callback"
        );
        let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
        for flag in [
            "HttpOnly",
            "Secure",
            "SameSite=Lax",
            "Max-Age=600",
            "Path=/auth/oidc/",
        ] {
            assert!(cookie.contains(flag));
        }
        (params, cookie.split(';').next().unwrap().into())
    }
    async fn finish(
        &self,
        state: &str,
        cookie: &str,
    ) -> warp::http::Response<warp::hyper::body::Bytes> {
        warp::test::request()
            .path(&format!(
                "/auth/oidc/example/callback?code=test-code&state={state}"
            ))
            .header("cookie", cookie)
            .reply(&self.routes())
            .await
    }
}

fn session_cookie(
    response: &warp::http::Response<warp::hyper::body::Bytes>,
    name: &str,
) -> Option<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|s| {
            s.strip_prefix(&format!("{name}="))
                .map(|s| s.split(';').next().unwrap().into())
        })
}

#[tokio::test(flavor = "multi_thread")]
async fn discovery_pkce_signed_login_and_existing_account_reuse() {
    let mock = MockProvider::new().await;
    let app = App::new(mock.configure("example"));
    let page = warp::test::request()
        .reply(&super::super::html(Arc::clone(&app.oidc)))
        .await;
    let page = std::str::from_utf8(page.body()).unwrap();
    assert!(page.contains("/auth/oidc/example"));
    assert!(page.contains("Example &amp; &lt;Friends&gt;"));
    assert!(
        page.contains("/auth/github")
            && page.contains("/auth/google")
            && page.contains("/auth/fusion")
    );
    let mut user_id = None;
    for _ in 0..2 {
        let (params, cookie) = app.begin().await;
        mock.token(&params["nonce"], false, |_| {});
        let response = app.finish(&params["state"], &cookie).await;
        assert_eq!(
            response.status(),
            StatusCode::SEE_OTHER,
            "{:?}",
            response.body()
        );
        assert_eq!(
            response.headers()[header::LOCATION],
            "https://game.example/lobby"
        );
        let token = session_cookie(&response, "AUTH_TOKEN").unwrap();
        let id = app.users.get_user_id(token).await.unwrap();
        if let Some(previous) = user_id {
            assert_eq!(id, previous);
        } else {
            user_id = Some(id);
        }
        let name = session_cookie(&response, "USER_NAME").unwrap();
        assert_eq!(
            percent_encoding::percent_decode_str(&name)
                .decode_utf8()
                .unwrap(),
            "Alice; ❤️\r\nTest"
        );
        assert_eq!(session_cookie(&response, STATE_COOKIE).unwrap(), "");
        let form = mock.state.lock().unwrap().forms.last().unwrap().clone();
        assert_eq!(form["code"], "test-code");
        assert_eq!(form["grant_type"], "authorization_code");
        assert_eq!(form["redirect_uri"], params["redirect_uri"]);
        assert_eq!(
            PkceCodeChallenge::from_code_verifier_sha256(&PkceCodeVerifier::new(
                form["code_verifier"].clone()
            ))
            .as_str(),
            params["code_challenge"]
        );
        assert!(mock
            .state
            .lock()
            .unwrap()
            .authorization
            .last()
            .unwrap()
            .as_ref()
            .unwrap()
            .starts_with("Basic "));
        assert_eq!(
            app.finish(&params["state"], &cookie).await.status(),
            StatusCode::BAD_REQUEST
        );
    }
    app.db
        .run_read_only(|tx| {
            assert_eq!(
                tx.query_row("SELECT count(*) FROM user", [], |row| row.get::<_, i64>(0))?,
                1
            );
            Ok(())
        })
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn rejects_wrong_issuer_audience_nonce_expiry_signature_and_access_token_hash() {
    let mock = MockProvider::new().await;
    let app = App::new(mock.configure("example"));
    for case in [
        "issuer",
        "audience",
        "nonce",
        "expiry",
        "signature",
        "access_hash",
        "authorized_party",
        "missing_token",
        "missing_nonce",
        "future_issued",
        "multiple_audiences",
    ] {
        let (params, cookie) = app.begin().await;
        mock.token(&params["nonce"], case == "signature", |claims| match case {
            "issuer" => claims["iss"] = json!("https://attacker.example"),
            "audience" => claims["aud"] = json!("different-client"),
            "nonce" => claims["nonce"] = json!("wrong-nonce"),
            "expiry" => claims["exp"] = json!(1),
            "access_hash" => claims["at_hash"] = json!("not-the-access-token-hash"),
            "authorized_party" => claims["azp"] = json!("different-client"),
            "missing_nonce" => {
                claims.as_object_mut().unwrap().remove("nonce");
            }
            "future_issued" => claims["iat"] = json!(claims["iat"].as_u64().unwrap() + 3600),
            "multiple_audiences" => claims["aud"] = json!(["test-client", "other-client"]),
            _ => {}
        });
        if case == "missing_token" {
            mock.state.lock().unwrap().token.clear();
        }
        let response = app.finish(&params["state"], &cookie).await;
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "{case}: {:?}",
            response.body()
        );
        assert!(session_cookie(&response, "AUTH_TOKEN").is_none());
        assert_eq!(session_cookie(&response, STATE_COOKIE).unwrap(), "");
        assert_eq!(
            app.finish(&params["state"], &cookie).await.status(),
            StatusCode::BAD_REQUEST
        );
    }
    app.db
        .run_read_only(|tx| {
            assert_eq!(
                tx.query_row("SELECT count(*) FROM user", [], |row| row.get::<_, i64>(0))?,
                0
            );
            Ok(())
        })
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn binds_browser_state_and_provider_and_expires_or_consumes_denials() {
    let mock = MockProvider::new().await;
    let app = App::new(mock.configure("example"));
    let (params, cookie) = app.begin().await;
    mock.token(&params["nonce"], false, |_| {});
    for (state, cookies) in [
        ("invented", cookie.as_str()),
        (params["state"].as_str(), "OIDC_STATE=wrong"),
        (params["state"].as_str(), ""),
    ] {
        assert_eq!(
            app.finish(state, cookies).await.status(),
            StatusCode::BAD_REQUEST
        );
    }
    let wrong_provider = warp::test::request()
        .path(&format!(
            "/auth/oidc/other/callback?code=test-code&state={}",
            params["state"]
        ))
        .header("cookie", &cookie)
        .reply(&app.routes())
        .await;
    assert_eq!(wrong_provider.status(), StatusCode::BAD_REQUEST);
    assert!(mock.state.lock().unwrap().forms.is_empty());
    // Invalid callbacks must not consume the legitimate browser's transaction.
    assert_eq!(
        app.finish(&params["state"], &cookie).await.status(),
        StatusCode::SEE_OTHER
    );
    let (params, cookie) = app.begin().await;
    app.oidc
        .pending
        .lock()
        .await
        .get_mut(&params["state"])
        .unwrap()
        .expires = Instant::now() - Duration::from_secs(1);
    assert_eq!(
        app.finish(&params["state"], &cookie).await.status(),
        StatusCode::BAD_REQUEST
    );
    let (params, cookie) = app.begin().await;
    let denied = warp::test::request()
        .path(&format!(
            "/auth/oidc/example/callback?error=access_denied&state={}",
            params["state"]
        ))
        .header("cookie", &cookie)
        .reply(&app.routes())
        .await;
    assert_eq!(denied.status(), StatusCode::BAD_REQUEST);
    assert_eq!(session_cookie(&denied, STATE_COOKIE).unwrap(), "");
    assert_eq!(
        app.finish(&params["state"], &cookie).await.status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn refreshes_keys_and_checks_userinfo_subject_with_post_auth() {
    let mock = MockProvider::new().await;
    let mut config = mock.configure("example");
    config.get_mut("example").unwrap().client_auth_method = ClientAuthMethod::Post;
    let app = App::new(config);
    let (params, cookie) = app.begin().await;
    mock.state.lock().unwrap().key =
        serde_json::to_value(signing_key(true).as_verification_key()).unwrap();
    mock.token(&params["nonce"], true, |claims| {
        claims.as_object_mut().unwrap().remove("name");
    });
    let response = app.finish(&params["state"], &cookie).await;
    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "{:?}",
        response.body()
    );
    let name = session_cookie(&response, "USER_NAME").unwrap();
    assert_eq!(
        percent_encoding::percent_decode_str(&name)
            .decode_utf8()
            .unwrap(),
        "UserInfo Alice"
    );
    {
        let state = mock.state.lock().unwrap();
        assert_eq!(state.forms[0]["client_secret"], "test-secret");
        assert_eq!(state.forms[0]["client_id"], "test-client");
        assert!(state.authorization[0].is_none());
    }
    let (params, cookie) = app.begin().await;
    mock.token(&params["nonce"], true, |claims| {
        claims.as_object_mut().unwrap().remove("name");
    });
    mock.state.lock().unwrap().userinfo["sub"] = json!("mallory");
    assert_eq!(
        app.finish(&params["state"], &cookie).await.status(),
        StatusCode::BAD_REQUEST
    );
    mock.state.lock().unwrap().wrong_discovery_issuer = true;
    assert_eq!(
        warp::test::request()
            .path("/auth/oidc/example")
            .reply(&app.routes())
            .await
            .status(),
        StatusCode::BAD_GATEWAY
    );
}

#[test]
fn config_is_optional_and_rejects_unsafe_urls_ids_and_missing_secrets() {
    let config: crate::Config = serde_json::from_value(json!({"db_path":"test.db", "external_uri":"http://localhost:7380", "port":7380,
        "fusion":{"client_id":"", "client_secret":""}, "google":{"client_id":"", "client_secret":""}, "github":{"client_id":"", "client_secret":""}})).unwrap();
    assert!(config.oidc.is_empty());
    let mut providers: BTreeMap<String, ProviderConfig> = serde_json::from_value(json!({"example":{
        "display_name":"Example", "issuer":"https://issuer.example", "client_id":"client", "client_secret":"secret"
    }})).unwrap();
    assert!(Oidc::new(&providers, "https://game.example").is_ok());
    assert!(Oidc::new(&providers, "http://game.example").is_err());
    for issuer in [
        "http://issuer.example",
        "https://user:password@issuer.example",
        "https://issuer.example#fragment",
        "https://issuer.example?query",
    ] {
        providers.get_mut("example").unwrap().issuer = issuer.into();
        assert!(Oidc::new(&providers, "https://game.example").is_err());
    }
    providers.get_mut("example").unwrap().issuer = "http://127.0.0.1:8080".into();
    assert!(Oidc::new(&providers, "http://localhost:7380").is_ok());
    let mut bad = providers.clone();
    bad.insert("../escape".into(), bad["example"].clone());
    assert!(Oidc::new(&bad, "https://game.example").is_err());
    let config = providers.get_mut("example").unwrap();
    config.client_secret = None;
    config.client_secret_env = Some("TURBO_HEARTS_TEST_MISSING_OIDC_SECRET_635ea1".into());
    assert!(Oidc::new(&providers, "https://game.example").is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn public_client_uses_pkce_and_issuer_namespaces_accounts() {
    let mock = MockProvider::new().await;
    let mut config = mock.configure("example");
    config.get_mut("example").unwrap().client_secret = None;
    let app = App::new(config);
    let (params, cookie) = app.begin().await;
    mock.token(&params["nonce"], false, |_| {});
    let response = app.finish(&params["state"], &cookie).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let state = mock.state.lock().unwrap();
    assert_eq!(state.forms[0]["client_id"], "test-client");
    assert!(!state.forms[0].contains_key("client_secret"));
    assert!(state.authorization[0].is_none());
    app.db
        .run_read_only(|tx| {
            let realm: String = tx.query_row("SELECT realm FROM user", [], |row| row.get(0))?;
            assert_eq!(realm, format!("oidc:example:{}", mock.issuer));
            Ok(())
        })
        .unwrap();
}

#[tokio::test]
async fn transport_rejects_redirects_insecure_endpoints_and_oversized_responses() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let location = base.clone() + "/target";
    let routes = warp::path("redirect")
        .map(move || {
            Response::builder()
                .status(302)
                .header(header::LOCATION, &location)
                .body(String::new())
                .unwrap()
        })
        .or(warp::path("target").map(|| "should not be followed"))
        .or(warp::path("large").map(|| "x".repeat(MAX_RESPONSE_BYTES + 1)));
    let task = tokio::spawn(warp::serve(routes).incoming(listener).run());
    let oidc = Oidc::new(&BTreeMap::new(), "http://localhost:7380").unwrap();
    let transport = Transport {
        client: oidc.http.clone(),
        local: true,
    };
    let request = |path: &str| {
        http::Request::builder()
            .uri(format!("{base}/{path}"))
            .body(Vec::new())
            .unwrap()
    };
    assert_eq!(
        transport.call(request("redirect")).await.unwrap().status(),
        StatusCode::FOUND
    );
    assert!(matches!(
        transport.call(request("large")).await,
        Err(TransportError::TooLarge)
    ));
    let transport = Transport {
        client: oidc.http.clone(),
        local: false,
    };
    assert!(matches!(
        transport.call(request("target")).await,
        Err(TransportError::Insecure)
    ));
    task.abort();
}
