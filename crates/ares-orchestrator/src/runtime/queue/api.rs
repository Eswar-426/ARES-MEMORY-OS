use super::{dto::EnqueueRequest, service::QueueService};
use ares_core::AresError;
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

fn map_err(e: AresError) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::from_u16(e.ipc_code() as u16)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        e.to_string(),
    )
}

pub struct QueueApiState {
    pub service: QueueService,
}

#[utoipa::path(
    post,
    path = "/api/v1/orchestrator/queue",
    request_body = EnqueueRequest,
    responses(
        (status = 200, description = "Item enqueued", body = super::models::WorkflowQueueItem),
        (status = 409, description = "Conflict - Duplicate execution_key")
    )
)]
pub async fn enqueue_workflow(
    State(state): State<Arc<QueueApiState>>,
    Json(payload): Json<EnqueueRequest>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let item = state.service.enqueue(payload).map_err(map_err)?;
    Ok(Json(item))
}
