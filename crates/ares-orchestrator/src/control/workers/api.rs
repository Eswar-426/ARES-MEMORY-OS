use super::{dto::*, service::WorkerService};
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

pub struct WorkersApiState {
    pub service: WorkerService,
}

#[utoipa::path(
    post,
    path = "/api/v1/orchestrator/workers",
    request_body = WorkerRegistrationRequest,
    responses(
        (status = 200, description = "Worker registered", body = Worker)
    )
)]
pub async fn register_worker(
    State(state): State<Arc<WorkersApiState>>,
    Json(payload): Json<WorkerRegistrationRequest>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let worker = state.service.register_worker(payload).map_err(map_err)?;
    Ok(Json(worker))
}

#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/workers",
    responses(
        (status = 200, description = "List workers", body = Vec<Worker>)
    )
)]
pub async fn list_workers(
    State(state): State<Arc<WorkersApiState>>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let workers = state.service.list_workers().map_err(map_err)?;
    Ok(Json(workers))
}

#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/workers/{id}",
    params(
        ("id" = String, Path, description = "Worker ID")
    ),
    responses(
        (status = 200, description = "Get worker", body = Worker)
    )
)]
pub async fn get_worker(
    State(state): State<Arc<WorkersApiState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    if let Some(worker) = state.service.get_worker(&id).map_err(map_err)? {
        Ok(Json(worker))
    } else {
        Err(map_err(AresError::validation("Worker not found")))
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/orchestrator/workers/{id}",
    params(
        ("id" = String, Path, description = "Worker ID")
    ),
    responses(
        (status = 200, description = "Delete worker")
    )
)]
pub async fn delete_worker(
    State(state): State<Arc<WorkersApiState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    state.service.delete_worker(&id).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}
