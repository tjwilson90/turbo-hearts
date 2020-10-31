use crate::{auth::RedirectToAuthChooser, config::CONFIG};
use http::{header, Response, StatusCode};
use std::convert::Infallible;
use turbo_hearts_api::CardsError;
use warp::{reject::Reject, Rejection, Reply};

#[derive(Debug)]
pub struct CardsReject(pub CardsError);

impl Reject for CardsReject {}

impl From<CardsReject> for Rejection {
    fn from(err: CardsReject) -> Self {
        warp::reject::custom(err)
    }
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    Ok(if let Some(_) = err.find::<RedirectToAuthChooser>() {
        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, format!("{}/auth", &CONFIG.external_uri))
            .body(String::new())
            .unwrap()
    } else if let Some(reject) = err.find::<CardsReject>() {
        let status = match reject.0 {
            CardsError::Serde { .. } | CardsError::Sqlite { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            CardsError::UnknownGame { .. } => StatusCode::NOT_FOUND,
            CardsError::UnknownAuthToken(_) => StatusCode::UNAUTHORIZED,
            _ => StatusCode::BAD_REQUEST,
        };
        Response::builder()
            .status(status)
            .body(reject.0.to_string())
            .unwrap()
    } else if err.is_not_found() {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(String::new())
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("{:?}", err))
            .unwrap()
    })
}
