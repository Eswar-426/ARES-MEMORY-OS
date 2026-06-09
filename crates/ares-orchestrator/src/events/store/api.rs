use crate::events::envelope::EventEnvelope;
use crate::events::store::service::EventStoreService;
use axum::{routing::post, Extension, Json, Router};
use std::sync::Arc;

pub struct EventStoreApiState {
    pub service: Arc<EventStoreService>,
}

pub fn router() -> Router<Arc<EventStoreApiState>> {
    Router::new().route("/publish", post(publish_event))
}

async fn publish_event(
    Extension(state): Extension<Arc<EventStoreApiState>>,
    Json(event): Json<EventEnvelope>,
) -> Json<serde_json::Value> {
    match state.service.append(&event) {
        Ok(_) => Json(serde_json::json!({"status": "success"})),
        Err(e) => Json(serde_json::json!({"status": "error", "message": e.to_string()})),
    }
}
