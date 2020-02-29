use crate::{
    auth,
    types::{Event, UserId},
    user::{User, Users},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::{
    stream::{Stream, StreamExt},
    sync::mpsc::UnboundedReceiver,
};
use warp::{sse, sse::ServerSentEvent, Filter, Rejection, Reply};

pub fn assets() -> reply!() {
    warp::path("assets")
        .and(auth::redirect_if_necessary())
        .untuple_one()
        .and(warp::fs::dir("./assets"))
}

pub fn users(users: infallible!(Users)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        ids: Vec<UserId>,
    }

    #[derive(Debug, Serialize)]
    struct Response {
        users: HashSet<User>,
    }

    async fn handle(users: Users, request: Request) -> Result<impl Reply, Rejection> {
        let Request { ids } = request;
        let users = users.get_users(ids).await?;
        Ok(warp::reply::json(&users))
    }

    warp::path!("users")
        .and(warp::post())
        .and(users)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn as_stream<E>(
    rx: UnboundedReceiver<E>,
) -> impl Stream<Item = Result<impl ServerSentEvent, warp::Error>>
where
    E: Event + Serialize + Send + Sync + 'static,
{
    rx.map(|event| {
        Ok(if event.is_ping() {
            sse::comment(String::new()).into_a()
        } else {
            sse::json(event).into_b()
        })
    })
}
