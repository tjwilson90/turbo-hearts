use crate::{config::CONFIG, db::Database, error::CardsError};
use http::{header, Response, StatusCode};
use reqwest::Client;
use rusqlite::{OptionalExtension, ToSql};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;
use warp::{path::FullPath, reject::Reject, Filter, Rejection, Reply};

#[derive(Clone)]
pub struct Users {
    db: Database,
    cache: Arc<Mutex<HashMap<String, String>>>,
}

impl Users {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get(&self, id: String) -> Result<String, CardsError> {
        let cache = self.cache.lock().await;
        if let Some(name) = cache.get(&id) {
            return Ok(name.clone());
        }
        drop(cache);
        match self.db.run_read_only(|tx| {
            tx.query_row("SELECT name FROM user WHERE id = ?", &[&id], |row| {
                row.get::<_, String>(0)
            })
            .optional()
            .map_err(|e| e.into())
        }) {
            Ok(name) => match name {
                Some(name) => {
                    let mut cache = self.cache.lock().await;
                    cache.insert(id, name.clone());
                    Ok(name)
                }
                None => Err(CardsError::UnknownPlayer(id)),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn put(&self, id: String, name: String) -> Result<(), CardsError> {
        self.db.run_with_retry(|tx| {
            tx.execute::<&[&dyn ToSql]>(
                "INSERT INTO user (id, name) VALUES (?, ?)",
                &[&id, &name],
            )?;
            Ok(())
        })?;
        let mut cache = self.cache.lock().await;
        cache.insert(id, name);
        Ok(())
    }
}

#[derive(Debug)]
pub struct AuthFlow(pub FullPath);

impl Reject for AuthFlow {}

impl From<AuthFlow> for Rejection {
    fn from(auth_flow: AuthFlow) -> Self {
        warp::reject::custom(auth_flow)
    }
}

impl AuthFlow {
    pub fn to_reply(&self) -> impl Reply {
        let state = Uuid::new_v4().to_string();
        let mut location = Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
        location
            .query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &CONFIG.client_id)
            .append_pair("redirect_uri", &CONFIG.redirect_uri())
            .append_pair("scope", "openid profile")
            .append_pair("state", &state)
            .append_pair("nonce", &Uuid::new_v4().to_string())
            .finish();
        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, location.to_string())
            .header(
                header::SET_COOKIE,
                format!("state={}:{}; Max-Age=600; HttpOnly", state, self.0.as_str()),
            )
            .body("")
            .unwrap()
    }
}

pub fn auth_redirect(
    users: impl Filter<Extract = (Users,), Error = Infallible> + Clone + Send,
    http_client: impl Filter<Extract = (Client,), Error = Infallible> + Clone + Send,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + Send {
    #[derive(Debug, Deserialize)]
    struct QueryParams {
        code: String,
        state: String,
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
        name: String,
    }

    async fn handle(
        users: Users,
        client: Client,
        query: QueryParams,
        state_cookie: String,
    ) -> Result<impl Reply, Rejection> {
        let mut split = state_cookie.splitn(2, ":");
        let state = split.next().unwrap();
        let original_uri = split.next().unwrap();
        if state != &query.state {
            return Err(warp::reject());
        }
        let response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&AuthRequest {
                grant_type: "authorization_code",
                code: &query.code,
                client_id: &CONFIG.client_id,
                client_secret: &CONFIG.client_secret,
                redirect_uri: &CONFIG.redirect_uri(),
            })
            .send()
            .await
            .unwrap();
        let response = response.json::<AuthResponse>().await.unwrap();
        let jwt = base64::decode(response.id_token.split(".").nth(1).unwrap()).unwrap();
        let jwt = serde_json::from_slice::<Jwt>(&jwt).unwrap();
        let token = Uuid::new_v4();
        users.put(token.to_string(), jwt.name.clone()).await?;
        Ok(Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, original_uri)
            .header(header::SET_COOKIE, format!("AUTH_TOKEN={}", token))
            .header(header::SET_COOKIE, format!("NAME={}", jwt.name))
            .body("")
            .unwrap())
    }

    warp::path!("redirect")
        .and(warp::get())
        .and(users)
        .and(http_client)
        .and(warp::query())
        .and(warp::cookie("state"))
        .and_then(handle)
}
