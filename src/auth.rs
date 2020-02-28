use crate::{
    auth::google::Google,
    user::{User, Users},
};
use http::{header, Response, StatusCode};
use reqwest::{Client, Url};
use serde::Deserialize;
use uuid::Uuid;
use warp::{path::FullPath, reject::Reject, Filter, Rejection, Reply};

mod google;

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
        let redirect = auth_url("google", &state);
        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, redirect.to_string())
            .header(
                header::SET_COOKIE,
                format!("state={}:{}; Max-Age=600; HttpOnly", state, self.0.as_str()),
            )
            .body("")
            .unwrap()
    }
}

pub fn auth_redirect(users: infallible!(Users), http_client: infallible!(Client)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct QueryParams {
        code: String,
        state: String,
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

        let user = exchange_code("google", &client, &query.code).await;

        let token = Uuid::new_v4();
        let response = Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, original_uri)
            .header(header::SET_COOKIE, format!("AUTH_TOKEN={}", token))
            .header(header::SET_COOKIE, format!("USER_ID={}", user.id))
            .header(header::SET_COOKIE, format!("USER_NAME={}", user.name))
            .body("")
            .unwrap();
        users.insert(token.to_string(), user).await?;
        Ok(response)
    }

    warp::path!("redirect")
        .and(warp::get())
        .and(users)
        .and(http_client)
        .and(warp::query())
        .and(warp::cookie("state"))
        .and_then(handle)
}

fn auth_url(kind: &str, state: &str) -> Url {
    match kind {
        Google::KIND => Google.auth_url(state),
        _ => panic!("Unknown auth provider: {}", kind),
    }
}

async fn exchange_code(kind: &str, http_client: &Client, code: &str) -> User {
    match kind {
        Google::KIND => Google.exchange_code(http_client, code).await,
        _ => panic!("Unknown auth provider: {}", kind),
    }
}
