use crate::{auth_redirect, judgment, CardsError, Database};
use http::{header, StatusCode};
use serde_json::json;
use std::{convert::Infallible, sync::Arc};
use turbo_hearts_api::UserId;
use warp::{Filter, Rejection, Reply};

fn response(body: impl Reply) -> warp::reply::Response {
    let mut response = body.into_response();
    let h = response.headers_mut();
    h.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    h.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    h.insert(header::REFERRER_POLICY, "same-origin".parse().unwrap());
    h.insert(header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'".parse().unwrap());
    response
}
async fn recover(err: Rejection) -> Result<warp::reply::Response, Infallible> {
    if err.find::<crate::RedirectToAuthChooser>().is_some() {
        return Ok(response(warp::redirect::see_other(http::Uri::from_static(
            "/auth",
        ))));
    }
    let (status, message) =
        if let Some(CardsError::Judgment { status, message }) = err.find::<CardsError>() {
            (*status, message.as_str())
        } else if err.find::<warp::reject::MissingCookie>().is_some()
            || err.find::<warp::reject::InvalidHeader>().is_some_and(|h| h.name() == "cookie")
            || err.find::<warp::reject::MissingHeader>().is_some_and(|h| h.name() == "cookie")
            || matches!(
                err.find::<CardsError>(),
                Some(CardsError::UnknownAuthToken(_))
            )
        {
            (
                401,
                "Please sign in again, then return to the decision lab.",
            )
        } else if err
            .find::<warp::filters::body::BodyDeserializeError>()
            .is_some()
            || err.find::<warp::reject::InvalidHeader>().is_some()
        {
            (400, "Invalid request.")
        } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
            (413, "Request too large.")
        } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
            (415, "Expected JSON.")
        } else if err.is_not_found() {
            (404, "Not found.")
        } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
            (405, "Method not allowed.")
        } else {
            log::error!("Judgment request failed: {:?}", err);
            (
                500,
                "Could not save or load this judgment. Please try again.",
            )
        };
    Ok(response(warp::reply::with_status(
        warp::reply::json(&json!({"error":message})),
        StatusCode::from_u16(status).unwrap(),
    )))
}

pub fn router(
    db: infallible!(&'static Database),
    user: rejection!(UserId),
    external_uri: &str,
) -> reply!() {
    let origin = Arc::new(
        reqwest::Url::parse(external_uri)
            .expect("external_uri URL")
            .origin()
            .ascii_serialization(),
    );
    // Both endpoints mutate state (loading records exposure). Require a same-origin POST
    // and a custom header, regardless of the older server-wide CORS policy.
    let gate = warp::post()
        .and(warp::header::optional::<String>("origin"))
        .and(warp::header::optional::<String>("x-judgment-request"))
        .and_then(move |sent: Option<String>, marker: Option<String>| {
            let origin = origin.clone();
            async move {
                if sent.as_deref() != Some(origin.as_str()) || marker.as_deref() != Some("1") {
                    Err(warp::reject::custom(judgment::error(
                        403,
                        "Open the decision lab on this site to submit judgments.",
                    )))
                } else {
                    Ok(())
                }
            }
        })
        .untuple_one();
    async fn state(
        db: &Database,
        user: UserId,
        q: judgment::Query,
    ) -> Result<impl Reply, Rejection> {
        let value = judgment::state(db, user, q)?;
        Ok(response(warp::reply::json(&value)))
    }
    async fn review(
        db: &Database,
        user: UserId,
        b: judgment::Review,
    ) -> Result<impl Reply, Rejection> {
        let value = judgment::review(db, user, b)?;
        Ok(response(warp::reply::json(&value)))
    }
    let state = warp::path!("api" / "state")
        .and(gate.clone())
        .and(db.clone())
        .and(user.clone())
        .and(warp::body::content_length_limit(4096))
        .and(warp::body::json())
        .and_then(state);
    let review = warp::path!("api" / "review")
        .and(gate)
        .and(db)
        .and(user)
        .and(warp::body::content_length_limit(32768))
        .and(warp::body::json())
        .and_then(review);
    let html = warp::path::end()
        .and(warp::get())
        .and(auth_redirect())
        .untuple_one()
        .map(|| {
            response(warp::reply::html(
                include_str!("../../scripts/judgment_ui/index.html")
                    .replace("/style.css", "/judge/style.css")
                    .replace("/app.js", "/judge/app.js"),
            ))
        });
    let js = warp::path!("app.js").and(warp::get()).map(|| {
        response(warp::reply::with_header(
            include_str!("../../scripts/judgment_ui/app.js"),
            header::CONTENT_TYPE,
            "text/javascript; charset=utf-8",
        ))
    });
    let css = warp::path!("style.css").and(warp::get()).map(|| {
        response(warp::reply::with_header(
            include_str!("../../scripts/judgment_ui/style.css"),
            header::CONTENT_TYPE,
            "text/css; charset=utf-8",
        ))
    });
    warp::path("judge").and(html.or(js).or(css).or(state).or(review).recover(recover))
}

#[cfg(test)]
mod tests;
