use crate::{ExternalUser, CONFIG};
use http::header;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

pub fn auth_url(state: &str) -> Url {
    let mut url = Url::parse("https://github.com/login/oauth/authorize").unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", &CONFIG.github.client_id)
        .append_pair("redirect_uri", &CONFIG.redirect_uri())
        .append_pair("state", &state)
        .finish();
    url
}

pub async fn exchange_code(http_client: &Client, code: &str, state: &str) -> ExternalUser {
    let response = http_client
        .post("https://github.com/login/oauth/access_token")
        .header(header::ACCEPT, "application/json")
        .json(&AuthRequest {
            state: &state,
            code: &code,
            client_id: &CONFIG.github.client_id,
            client_secret: &CONFIG.github.client_secret,
            redirect_uri: &CONFIG.redirect_uri(),
        })
        .send()
        .await
        .unwrap();
    let response = response.json::<AuthResponse>().await.unwrap();
    let user = http_client
        .get("https://api.github.com/user")
        .header(
            header::AUTHORIZATION,
            format!("token {}", response.access_token),
        )
        .header(header::USER_AGENT, "Turbo-Hearts")
        .send()
        .await
        .unwrap();
    let User { login, id } = user.json().await.unwrap();
    ExternalUser {
        name: login,
        realm: "github".to_string(),
        external_id: id.to_string(),
    }
}

#[derive(Debug, Serialize)]
struct AuthRequest<'a> {
    state: &'a str,
    code: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
    redirect_uri: &'a str,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct User {
    login: String,
    id: u64,
}
