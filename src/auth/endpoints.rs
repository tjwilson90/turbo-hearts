use crate::{auth::google, config::CONFIG, user::Users};
use http::{header, Response, StatusCode};
use reqwest::Client;
use serde::Deserialize;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{Filter, Rejection, Reply};

pub fn router(users: infallible!(Users), http_client: infallible!(Client)) -> reply!() {
    redirect(users, http_client)
        .or(warp::path("auth").and(html().or(google())))
        .boxed()
}

fn html() -> reply!() {
    warp::path::end()
        .and(warp::get())
        .and(warp::fs::file("auth.html"))
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

fn redirect(users: infallible!(Users), http_client: infallible!(Client)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct QueryParams {
        code: String,
        state: String,
    }

    async fn handle(
        users: Users,
        http_client: Client,
        query: QueryParams,
        state_cookie: String,
    ) -> Result<impl Reply, Rejection> {
        let mut split = state_cookie.splitn(2, ":");
        let provider = split.next().unwrap();
        let state = split.next().unwrap();
        if state != &query.state {
            return Err(warp::reject());
        }

        let user = match provider {
            "google" => google::exchange_code(&http_client, &query.code).await,
            _ => panic!("Unknown provider: {}", provider),
        };
        let token = Uuid::new_v4();
        let user = users.insert(token.to_string(), user).await?;

        let response = Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, format!("{}/lobby", &CONFIG.external_uri))
            .header(header::SET_COOKIE, format!("AUTH_TOKEN={}", token))
            .header(header::SET_COOKIE, format!("USER_ID={}", user.id))
            .header(header::SET_COOKIE, format!("USER_NAME={}", user.name))
            .body("")
            .unwrap();
        Ok(response)
    }

    warp::path!("redirect")
        .and(warp::get())
        .and(users)
        .and(http_client)
        .and(warp::query())
        .and(warp::cookie("STATE"))
        .and_then(handle)
}
