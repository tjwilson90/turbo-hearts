use warp::{reject, reject::Reject, Filter, Rejection};

pub mod endpoints;
pub mod fusion;
pub mod github;
pub mod google;

#[derive(Debug)]
pub struct RedirectToAuthChooser;

impl Reject for RedirectToAuthChooser {}

pub fn redirect_if_necessary() -> rejection!(()) {
    async fn handle(auth_token: Option<String>) -> Result<(), Rejection> {
        match auth_token {
            Some(_) => Ok(()),
            None => Err(reject::custom(RedirectToAuthChooser)),
        }
    }

    warp::cookie::optional("AUTH_TOKEN").and_then(handle)
}
