use super::service::ExecutionService;
use ares_core::AresError;
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

fn map_err(e: AresError) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::from_u16(e.ipc_code() as u16)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        e.to_string(),
    )
}

pub struct ExecutionApiState {
    pub service: ExecutionService,
}

#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/executions/distributed",
    responses(
        (status = 200, description = "List executions", body = Vec<DistributedExecution>)
    )
)]
pub async fn list_distributed_executions(
    State(state): State<Arc<ExecutionApiState>>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let items = state.service.list_executions(100).map_err(map_err)?; // Default limit
    Ok(Json(items))
}
