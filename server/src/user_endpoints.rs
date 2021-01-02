use crate::{User, Users};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use turbo_hearts_api::UserId;
use warp::{Filter, Rejection, Reply};

pub fn router<'a>(users: infallible!(&'a Users)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        ids: Vec<UserId>,
    }

    #[derive(Debug, Serialize)]
    struct Response {
        users: HashSet<User>,
    }

    async fn handle(users: &Users, request: Request) -> Result<impl Reply, Rejection> {
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
