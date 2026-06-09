use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::time::Duration;
use tokio_stream::StreamExt as _;

pub struct SseApiState {
    // Shared state like a broadcast receiver channel would go here
}

pub fn router() -> Router<Arc<SseApiState>> {
    Router::new().route("/sse", get(sse_handler))
}

async fn sse_handler(State(_state): State<Arc<SseApiState>>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // A placeholder stream that sends a heartbeat ping every 10 seconds.
    // In a real implementation, this would yield events from a channel.
    let stream = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(10)))
        .map(|_| Ok(Event::default().data("ping")));

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)).text("keep-alive-text"))
}
