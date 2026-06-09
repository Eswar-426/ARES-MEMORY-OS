use super::service::DlqService;
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

pub struct DlqApiState {
    pub service: DlqService,
}

#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/dlq",
    responses(
        (status = 200, description = "List dead letter queue", body = Vec<super::models::DeadLetterItem>)
    )
)]
pub async fn list_dlq(
    State(state): State<Arc<DlqApiState>>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let items = state.service.list_dlq(100).map_err(map_err)?; // Default limit
    Ok(Json(items))
}
