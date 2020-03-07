use crate::{config::CONFIG, user::ExternalUser};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

pub fn auth_url(state: &str) -> Url {
    let mut url = Url::parse("https://auth.anti.run/oauth2/authorize").unwrap();
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &CONFIG.fusion.client_id)
        .append_pair("redirect_uri", &CONFIG.redirect_uri())
        .append_pair("tenantId", "2378557e-9044-3e5b-e990-f36cc7f6b6a3")
        .append_pair("state", &state)
        .append_pair("scope", "openid")
        .finish();
    url
}

pub async fn exchange_code(http_client: &Client, code: &str) -> ExternalUser {
    let response = http_client
        .post("https://auth.anti.run/oauth2/token")
        .form(&AuthRequest {
            grant_type: "authorization_code",
            code: &code,
            client_id: &CONFIG.fusion.client_id,
            client_secret: &CONFIG.fusion.client_secret,
            redirect_uri: &CONFIG.redirect_uri(),
        })
        .send()
        .await
        .unwrap();
    let response = response.json::<AuthResponse>().await.unwrap();
    let jwt = base64::decode(response.id_token.split(".").nth(1).unwrap()).unwrap();
    let Jwt {
        sub,
        preferred_username,
    } = serde_json::from_slice::<Jwt>(&jwt).unwrap();
    ExternalUser {
        name: preferred_username,
        realm: "fusion".to_string(),
        external_id: sub,
    }
}

#[derive(Debug, Serialize)]
struct AuthRequest<'a> {
    grant_type: &'a str,
    code: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
    redirect_uri: &'a str,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    id_token: String,
}

#[derive(Debug, Deserialize)]
struct Jwt {
    sub: String,
    preferred_username: String,
}
