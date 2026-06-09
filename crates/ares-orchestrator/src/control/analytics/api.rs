#[allow(unused_imports)]
use super::service::{AnalyticsService, OrchestratorAnalytics};
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

pub struct AnalyticsApiState {
    pub service: AnalyticsService,
}

#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/analytics",
    responses(
        (status = 200, description = "Orchestrator analytics", body = OrchestratorAnalytics)
    )
)]
pub async fn get_analytics(
    State(state): State<Arc<AnalyticsApiState>>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let analytics = state.service.get_analytics().map_err(map_err)?;
    Ok(Json(analytics))
}
