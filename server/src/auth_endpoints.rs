use crate::{ExternalUser, Users, CONFIG};
use http::{header, Response, StatusCode};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::Deserialize;
use std::{convert::Infallible, sync::Arc};
use uuid::Uuid;
use warp::{Filter, Rejection, Reply};

mod fusion;
mod github;
mod google;
pub(crate) mod oidc;

pub fn router(
    users: infallible!(&'static Users),
    http_client: infallible!(&'static Client),
    oidc: Arc<oidc::Oidc>,
) -> reply!() {
    oidc::router(users.clone(), Arc::clone(&oidc))
        .or(redirect(users, http_client))
        .or(warp::path("auth").and(html(oidc).or(fusion()).or(github()).or(google())))
        .boxed()
}

fn html(oidc: Arc<oidc::Oidc>) -> reply!() {
    warp::path::end().and(warp::get()).map(move || {
        warp::reply::html(
            include_str!("../../assets/auth/index.html")
                .replace("<!-- OIDC_PROVIDERS -->", &oidc.login_links()),
        )
    })
}

// Keep the existing browser-readable session cookies: the lobby clears them
// when an SSE session is rejected. Encode display names as expected by cookie.parse.
async fn complete_login(
    users: &Users,
    external_uri: &str,
    user: ExternalUser,
) -> Result<Response<String>, Rejection> {
    let token = Uuid::new_v4();
    let user = users.insert(token.to_string(), user).await?;
    let attrs = if external_uri.starts_with("https://") {
        "Path=/; SameSite=Lax; Secure"
    } else {
        "Path=/; SameSite=Lax"
    };
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(
            header::LOCATION,
            format!("{}/lobby", external_uri.trim_end_matches('/')),
        )
        .header(header::SET_COOKIE, format!("AUTH_TOKEN={token}; {attrs}"))
        .header(header::SET_COOKIE, format!("USER_ID={}; {attrs}", user.id))
        .header(
            header::SET_COOKIE,
            format!(
                "USER_NAME={}; {attrs}",
                utf8_percent_encode(&user.name, NON_ALPHANUMERIC)
            ),
        )
        .header(header::CACHE_CONTROL, "no-store")
        .header("Referrer-Policy", "no-referrer")
        .body(String::new())
        .unwrap())
}

fn fusion() -> reply!() {
    async fn handle() -> Result<impl Reply, Infallible> {
        let state = Uuid::new_v4().to_string();
        let redirect = fusion::auth_url(&state);
        Ok(Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, redirect.to_string())
            .header(
                header::SET_COOKIE,
                format!("STATE=fusion:{}; Path=/; HttpOnly; Max-Age=600", state),
            )
            .body("")
            .unwrap())
    }

    warp::path!("fusion").and(warp::get()).and_then(handle)
}

fn github() -> reply!() {
    async fn handle() -> Result<impl Reply, Infallible> {
        let state = Uuid::new_v4().to_string();
        let redirect = github::auth_url(&state);
        Ok(Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, redirect.to_string())
            .header(
                header::SET_COOKIE,
                format!("STATE=github:{}; Path=/; HttpOnly; Max-Age=600", state),
            )
            .body("")
            .unwrap())
    }

    warp::path!("github").and(warp::get()).and_then(handle)
}

fn google() -> reply!() {
    async fn handle() -> Result<impl Reply, Infallible> {
        let state = Uuid::new_v4().to_string();
        let redirect = google::auth_url(&state);
        Ok(Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, redirect.to_string())
            .header(
                header::SET_COOKIE,
                format!("STATE=google:{}; Path=/; HttpOnly; Max-Age=600", state),
            )
            .body("")
            .unwrap())
    }

    warp::path!("google").and(warp::get()).and_then(handle)
}

fn redirect<'a>(users: infallible!(&'a Users), http_client: infallible!(&'a Client)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct QueryParams {
        code: String,
        state: String,
    }

    async fn handle(
        users: &Users,
        http_client: &Client,
        query: QueryParams,
        state_cookie: String,
    ) -> Result<impl Reply, Rejection> {
        let (provider, state) = state_cookie
            .split_once(':')
            .ok_or_else(|| warp::reject::custom(oidc::LoginError::Invalid))?;
        if state != &query.state {
            return Err(warp::reject::custom(oidc::LoginError::Invalid));
        }

        let user = match provider {
            "fusion" => fusion::exchange_code(&http_client, &query.code).await,
            "github" => github::exchange_code(&http_client, &query.code, &query.state).await,
            "google" => google::exchange_code(&http_client, &query.code).await,
            _ => return Err(warp::reject::custom(oidc::LoginError::Invalid)),
        };
        complete_login(users, &CONFIG.external_uri, user).await
    }

    warp::path!("redirect")
        .and(warp::get())
        .and(users)
        .and(http_client)
        .and(warp::query())
        .and(warp::cookie("STATE"))
        .and_then(handle)
}
