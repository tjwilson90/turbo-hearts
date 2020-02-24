use crate::types::Event;
use serde::Serialize;
use tokio::{
    stream::{Stream, StreamExt},
    sync::mpsc::UnboundedReceiver,
};
use warp::{sse, sse::ServerSentEvent, Filter};

pub mod game;
pub mod lobby;

pub fn assets() -> reply!() {
    warp::path("assets")
        .and(crate::auth_flow())
        .untuple_one()
        .and(warp::fs::dir("./assets"))
}

fn as_stream<E>(
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
