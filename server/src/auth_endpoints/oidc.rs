//! Shared authorization-code OpenID Connect login. Provider metadata and keys
//! are rediscovered for each exchange so signing-key rotation needs no restart.
use crate::{ExternalUser, Users};
use futures_util::{future::BoxFuture, FutureExt};
use http::{header, Response, StatusCode};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata, CoreUserInfoClaims},
    AccessTokenHash, AsyncHttpClient, AuthType, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, EndpointMaybeSet, EndpointNotSet, EndpointSet, HttpRequest, HttpResponse, IssuerUrl,
    Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
};
use reqwest::{Client, Url};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tokio::sync::Mutex;
use warp::{Filter, Rejection, Reply};

const LOGIN_TTL: Duration = Duration::from_secs(600);
const MAX_PENDING: usize = 1024;
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const STATE_COOKIE: &str = "OIDC_STATE";
type DiscoveredClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderConfig {
    pub display_name: String,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_secret_env: Option<String>,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub client_auth_method: ClientAuthMethod,
}

impl std::fmt::Debug for ProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderConfig")
            .field("issuer", &self.issuer)
            .field("display_name", &self.display_name)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientAuthMethod {
    #[default]
    Basic,
    Post,
}

fn default_scopes() -> Vec<String> {
    vec!["profile".into()]
}

struct Provider {
    config: ProviderConfig,
    issuer: IssuerUrl,
    secret: Option<ClientSecret>,
    redirect: RedirectUrl,
}

struct PendingLogin {
    provider: String,
    browser: CsrfToken,
    nonce: Nonce,
    verifier: PkceCodeVerifier,
    expires: Instant,
}

pub struct Oidc {
    providers: BTreeMap<String, Provider>,
    http: Client,
    external_uri: String,
    login_path: String,
    secure_cookies: bool,
    pending: Mutex<HashMap<String, PendingLogin>>,
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("The login was invalid or expired. Please start again.")]
    Invalid,
    #[error("The identity provider is unavailable. Please try again.")]
    Unavailable,
    #[error("Too many logins are in progress. Please try again shortly.")]
    Busy,
}
impl warp::reject::Reject for LoginError {}
impl LoginError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Invalid => StatusCode::BAD_REQUEST,
            Self::Unavailable => StatusCode::BAD_GATEWAY,
            Self::Busy => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

#[derive(Debug, Error)]
enum TransportError {
    #[error("insecure provider endpoint")]
    Insecure,
    #[error("provider response too large")]
    TooLarge,
    #[error("provider request failed")]
    Network(#[from] reqwest::Error),
    #[error("invalid HTTP response")]
    Http(#[from] http::Error),
}

struct Transport {
    client: Client,
    local: bool,
}
impl<'c> AsyncHttpClient<'c> for Transport {
    type Error = TransportError;
    type Future = BoxFuture<'c, Result<HttpResponse, TransportError>>;
    fn call(&'c self, request: HttpRequest) -> Self::Future {
        self.request(request).boxed()
    }
}
impl Transport {
    // Restrict every back-channel request, including discovery's JWKS fetch.
    // Redirects are disabled so authorization codes cannot leak to another URL.
    async fn request(&self, request: HttpRequest) -> Result<HttpResponse, TransportError> {
        let url = Url::parse(&request.uri().to_string()).map_err(|_| TransportError::Insecure)?;
        if !safe_endpoint(&url, self.local) {
            return Err(TransportError::Insecure);
        }
        let (parts, body) = request.into_parts();
        let mut response = self
            .client
            .request(parts.method, url)
            .headers(parts.headers)
            .body(body)
            .send()
            .await?;
        let mut builder = Response::builder().status(response.status());
        *builder.headers_mut().unwrap() = response.headers().clone();
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > MAX_RESPONSE_BYTES {
                return Err(TransportError::TooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(builder.body(bytes)?)
    }
}

fn loopback(url: &Url) -> bool {
    match url.host() {
        Some(openidconnect::url::Host::Domain("localhost")) => true,
        Some(openidconnect::url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(openidconnect::url::Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    }
}

fn safe_endpoint(url: &Url, allow_loopback: bool) -> bool {
    url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && (url.scheme() == "https" || (allow_loopback && url.scheme() == "http" && loopback(url)))
}

impl Oidc {
    pub fn new(
        configs: &BTreeMap<String, ProviderConfig>,
        external_uri: &str,
    ) -> Result<Self, String> {
        let base = Url::parse(external_uri).map_err(|_| "invalid external_uri")?;
        if !configs.is_empty() && (!safe_endpoint(&base, true) || base.query().is_some()) {
            return Err(
                "OIDC requires an HTTPS external_uri (HTTP is allowed on loopback for development)"
                    .into(),
            );
        }
        let external_uri = external_uri.trim_end_matches('/').to_owned();
        let mut providers = BTreeMap::new();
        for (id, config) in configs {
            if id.is_empty()
                || id.len() > 64
                || !id
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_')
            {
                return Err("OIDC provider IDs must use 1–64 lowercase letters, digits, hyphens, or underscores".into());
            }
            let invalid = || format!("invalid OIDC configuration for {id}");
            let issuer = IssuerUrl::new(config.issuer.clone()).map_err(|_| invalid())?;
            if !safe_endpoint(issuer.url(), true)
                || issuer.url().query().is_some()
                || config.client_id.trim().is_empty()
                || config.display_name.trim().is_empty()
                || config.scopes.iter().any(|s| {
                    s.is_empty()
                        || !s.bytes().all(|b| {
                            b == 0x21 || (0x23..=0x5b).contains(&b) || (0x5d..=0x7e).contains(&b)
                        })
                })
            {
                return Err(invalid());
            }
            let secret = match (&config.client_secret, &config.client_secret_env) {
                (Some(_), Some(_)) => {
                    return Err(format!(
                        "{id}: configure only one of client_secret and client_secret_env"
                    ))
                }
                (Some(value), None) => Some(value.clone()),
                (None, Some(var)) => Some(
                    std::env::var(var)
                        .map_err(|_| format!("{id}: client_secret_env is not set"))?,
                ),
                (None, None) => None,
            };
            if secret.as_ref().is_some_and(|s| s.is_empty()) {
                return Err(invalid());
            }
            let redirect = RedirectUrl::new(format!("{external_uri}/auth/oidc/{id}/callback"))
                .map_err(|_| invalid())?;
            providers.insert(
                id.clone(),
                Provider {
                    config: config.clone(),
                    issuer,
                    secret: secret.map(ClientSecret::new),
                    redirect,
                },
            );
        }
        Ok(Self {
            providers,
            external_uri,
            login_path: format!("{}/auth/oidc/", base.path().trim_end_matches('/')),
            secure_cookies: base.scheme() == "https",
            http: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|_| "could not create OIDC HTTP client")?,
            pending: Mutex::new(HashMap::new()),
        })
    }

    async fn discover(&self, provider: &Provider) -> Result<DiscoveredClient, LoginError> {
        let local = loopback(provider.issuer.url());
        let http = Transport {
            client: self.http.clone(),
            local,
        };
        let metadata = CoreProviderMetadata::discover_async(provider.issuer.clone(), &http)
            .await
            .map_err(|_| LoginError::Unavailable)?;
        if !safe_endpoint(metadata.authorization_endpoint().url(), local)
            || !metadata
                .token_endpoint()
                .is_some_and(|url| safe_endpoint(url.url(), local))
        {
            return Err(LoginError::Unavailable);
        }
        Ok(CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(provider.config.client_id.clone()),
            provider.secret.clone(),
        )
        .set_redirect_uri(provider.redirect.clone())
        .set_auth_type(match provider.config.client_auth_method {
            ClientAuthMethod::Basic => AuthType::BasicAuth,
            ClientAuthMethod::Post => AuthType::RequestBody,
        }))
    }

    fn state_cookie(&self, value: &str, max_age: u64) -> String {
        format!(
            "{STATE_COOKIE}={value}; Path={}; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
            self.login_path,
            if self.secure_cookies { "; Secure" } else { "" }
        )
    }

    async fn begin(&self, id: &str) -> Result<Response<String>, Rejection> {
        let provider = self.providers.get(id).ok_or_else(warp::reject::not_found)?;
        let client = self
            .discover(provider)
            .await
            .map_err(warp::reject::custom)?;
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let mut request = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge);
        for scope in &provider.config.scopes {
            if scope != "openid" {
                request = request.add_scope(Scope::new(scope.clone()));
            }
        }
        let (url, state, nonce) = request.url();
        let browser = CsrfToken::new_random();
        let cookie = self.state_cookie(browser.secret(), LOGIN_TTL.as_secs());
        let now = Instant::now();
        let mut pending = self.pending.lock().await;
        pending.retain(|_, login| login.expires > now);
        if pending.len() >= MAX_PENDING {
            return Err(warp::reject::custom(LoginError::Busy));
        }
        pending.insert(
            state.secret().clone(),
            PendingLogin {
                provider: id.into(),
                browser,
                nonce,
                verifier,
                expires: now + LOGIN_TTL,
            },
        );
        Ok(Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, url.as_str())
            .header(header::SET_COOKIE, cookie)
            .header(header::CACHE_CONTROL, "no-store")
            .header("Referrer-Policy", "no-referrer")
            .body(String::new())
            .unwrap())
    }

    async fn take_login(
        &self,
        id: &str,
        query: &Callback,
        cookie: Option<String>,
    ) -> Result<PendingLogin, LoginError> {
        let state = query.state.as_ref().ok_or(LoginError::Invalid)?;
        let browser = CsrfToken::new(cookie.ok_or(LoginError::Invalid)?);
        let mut pending = self.pending.lock().await;
        pending.retain(|_, login| login.expires > Instant::now());
        let login = pending.get(state).ok_or(LoginError::Invalid)?;
        if login.provider != id || login.browser != browser {
            return Err(LoginError::Invalid);
        }
        // Consume even denials or failed exchanges; the browser must restart.
        Ok(pending.remove(state).unwrap())
    }

    async fn exchange(
        &self,
        id: &str,
        login: PendingLogin,
        code: String,
    ) -> Result<ExternalUser, LoginError> {
        let provider = self.providers.get(id).ok_or(LoginError::Invalid)?;
        let client = self.discover(provider).await?;
        let http = Transport {
            client: self.http.clone(),
            local: loopback(provider.issuer.url()),
        };
        let tokens = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|_| LoginError::Unavailable)?
            .set_pkce_verifier(login.verifier)
            .request_async(&http)
            .await
            .map_err(|_| LoginError::Invalid)?;
        let token = tokens.id_token().ok_or(LoginError::Invalid)?;
        let verifier = client
            .id_token_verifier()
            .set_issue_time_verifier_fn(|issued| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| "invalid system clock".to_owned())?
                    .as_secs() as i64;
                if issued.timestamp() > now + 60 {
                    Err("ID token was issued in the future".into())
                } else {
                    Ok(())
                }
            });
        let claims = token
            .claims(&verifier, &login.nonce)
            .map_err(|_| LoginError::Invalid)?;
        if claims
            .authorized_party()
            .is_some_and(|party| party.as_str() != provider.config.client_id)
            || (claims.audiences().len() > 1 && claims.authorized_party().is_none())
        {
            return Err(LoginError::Invalid);
        }
        if claims.subject().as_str().is_empty() {
            return Err(LoginError::Invalid);
        }
        if let Some(expected) = claims.access_token_hash() {
            let actual = AccessTokenHash::from_token(
                tokens.access_token(),
                token.signing_alg().map_err(|_| LoginError::Invalid)?,
                token
                    .signing_key(&verifier)
                    .map_err(|_| LoginError::Invalid)?,
            )
            .map_err(|_| LoginError::Invalid)?;
            if &actual != expected {
                return Err(LoginError::Invalid);
            }
        }
        let mut name = claims
            .name()
            .and_then(|names| names.get(None))
            .map(|s| s.to_string())
            .or_else(|| claims.preferred_username().map(|s| s.to_string()));
        if name.as_ref().is_none_or(|s| s.trim().is_empty()) {
            if let Ok(request) = client.user_info(
                tokens.access_token().clone(),
                Some(claims.subject().clone()),
            ) {
                // The library checks that UserInfo's subject matches the ID token.
                let info: CoreUserInfoClaims = request
                    .request_async(&http)
                    .await
                    .map_err(|_| LoginError::Invalid)?;
                name = info
                    .name()
                    .and_then(|names| names.get(None))
                    .map(|s| s.to_string())
                    .or_else(|| info.preferred_username().map(|s| s.to_string()));
            }
        }
        Ok(ExternalUser {
            name: name
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| claims.subject().to_string()),
            // Include the exact issuer so changing a provider's configuration
            // cannot silently give a different issuer access to existing users.
            realm: format!("oidc:{id}:{}", provider.issuer.as_str()),
            external_id: claims.subject().to_string(),
        })
    }

    pub fn login_links(&self) -> String {
        self.providers
            .iter()
            .map(|(id, provider)| {
                format!(
                    "<div class=\"auth-source\"><a href=\"{}{id}\">Sign in with {}</a></div>",
                    escape_html(&self.login_path),
                    escape_html(&provider.config.display_name)
                )
            })
            .collect()
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Default, Deserialize)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

pub fn router(
    users: infallible!(&'static Users),
    oidc: Arc<Oidc>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let state = warp::any().map(move || Arc::clone(&oidc));
    let start = warp::path!("auth" / "oidc" / String)
        .and(warp::get())
        .and(state.clone())
        .and_then(|id: String, oidc: Arc<Oidc>| async move { oidc.begin(&id).await });
    let callback = warp::path!("auth" / "oidc" / String / "callback")
        .and(warp::get())
        .and(state)
        .and(users)
        .and(warp::query::<Callback>())
        .and(warp::cookie::optional(STATE_COOKIE))
        .and_then(
            |id: String,
             oidc: Arc<Oidc>,
             users: &'static Users,
             query: Callback,
             cookie: Option<String>| async move {
                let login = oidc
                    .take_login(&id, &query, cookie)
                    .await
                    .map_err(warp::reject::custom)?;
                let result = async {
                    if query.error.is_some() {
                        return Err(warp::reject::custom(LoginError::Invalid));
                    }
                    let code = query
                        .code
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| warp::reject::custom(LoginError::Invalid))?;
                    let user = oidc
                        .exchange(&id, login, code)
                        .await
                        .map_err(warp::reject::custom)?;
                    super::complete_login(users, &oidc.external_uri, user).await
                }
                .await;
                // Always clear the consumed transaction cookie, including denials.
                let mut response = match result {
                    Ok(response) => response.into_response(),
                    Err(error) => crate::error::handle_rejection(error)
                        .await
                        .unwrap()
                        .into_response(),
                };
                response.headers_mut().append(
                    header::SET_COOKIE,
                    oidc.state_cookie("", 0).parse().unwrap(),
                );
                Ok::<_, Rejection>(response)
            },
        );
    start.or(callback)
}

#[cfg(test)]
mod tests;
