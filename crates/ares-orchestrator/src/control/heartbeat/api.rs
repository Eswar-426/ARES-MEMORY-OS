use crate::control::workers::dto::WorkerStatusUpdateRequest;
use super::service::HeartbeatService;
use ares_core::AresError;
use axum::{
    extract::{Path, State},
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

pub struct HeartbeatApiState {
    pub service: HeartbeatService,
}

#[utoipa::path(
    post,
    path = "/api/v1/orchestrator/workers/{id}/heartbeat",
    request_body = WorkerStatusUpdateRequest,
    params(
        ("id" = String, Path, description = "Worker ID")
    ),
    responses(
        (status = 200, description = "Heartbeat processed")
    )
)]
pub async fn worker_heartbeat(
    State(state): State<Arc<HeartbeatApiState>>,
    Path(id): Path<String>,
    Json(payload): Json<WorkerStatusUpdateRequest>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    state.service.process_heartbeat(&id, payload.available_cpu, payload.available_memory).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
