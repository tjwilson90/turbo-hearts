use crate::auth;
use warp::Filter;

pub fn router() -> reply!() {
    warp::path("assets")
        .and(auth::redirect_if_necessary())
        .untuple_one()
        .and(warp::fs::dir("./assets"))
}
