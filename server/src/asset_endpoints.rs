use crate::auth_redirect;
use warp::Filter;

pub fn router() -> reply!() {
    warp::path("assets")
        .and(auth_redirect())
        .untuple_one()
        .and(warp::fs::dir("./assets"))
}
