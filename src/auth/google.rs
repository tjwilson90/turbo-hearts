use crate::{config::CONFIG, types::UserId, user::User};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Google;

impl Google {
    pub const KIND: &'static str = "google";

    pub fn auth_url(&self, state: &str) -> Url {
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &CONFIG.google.client_id)
            .append_pair("redirect_uri", &CONFIG.redirect_uri())
            .append_pair("scope", "openid profile")
            .append_pair("state", &state)
            .append_pair("nonce", &Uuid::new_v4().to_string())
            .finish();
        url
    }

    pub async fn exchange_code(&self, http_client: &Client, code: &str) -> User {
        let response = http_client
            .post("https://oauth2.googleapis.com/token")
            .form(&AuthRequest {
                grant_type: "authorization_code",
                code: &code,
                client_id: &CONFIG.google.client_id,
                client_secret: &CONFIG.google.client_secret,
                redirect_uri: &CONFIG.redirect_uri(),
            })
            .send()
            .await
            .unwrap();
        let response = response.json::<AuthResponse>().await.unwrap();
        let jwt = base64::decode(response.id_token.split(".").nth(1).unwrap()).unwrap();
        let Jwt { sub, name } = serde_json::from_slice::<Jwt>(&jwt).unwrap();
        User {
            id: UserId::new(),
            name,
            realm: "google".to_string(),
            external_id: sub,
        }
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
    name: String,
}
