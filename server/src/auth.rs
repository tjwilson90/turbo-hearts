use warp::{reject, reject::Reject, Filter, Rejection};

#[derive(Debug)]
pub struct RedirectToAuthChooser;

impl Reject for RedirectToAuthChooser {}

pub fn auth_redirect() -> rejection!(()) {
    async fn handle(auth_token: Option<String>) -> Result<(), Rejection> {
        match auth_token {
            Some(_) => Ok(()),
            None => Err(reject::custom(RedirectToAuthChooser)),
        }
    }

    warp::cookie::optional("AUTH_TOKEN").and_then(handle)
}
