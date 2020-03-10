use crate::{
    auth,
    user::{User, UserId, Users},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use warp::{Filter, Rejection, Reply};

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
