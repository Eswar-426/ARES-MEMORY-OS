use super::routing::DiscoveryService;
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

pub struct DiscoveryApiState {
    pub service: DiscoveryService,
}

#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/workers/discovery/{capability_name}",
    params(
        ("capability_name" = String, Path, description = "Capability Name")
    ),
    responses(
        (status = 200, description = "List matching workers", body = Vec<Worker>)
    )
)]
pub async fn discover_by_capability(
    State(state): State<Arc<DiscoveryApiState>>,
    Path(capability_name): Path<String>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let workers = state
        .service
        .find_workers_by_capability(&capability_name, None)
        .map_err(map_err)?;
    Ok(Json(workers))
}

#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/workers/discovery/{capability_name}/{capability_version}",
    params(
        ("capability_name" = String, Path, description = "Capability Name"),
        ("capability_version" = String, Path, description = "Capability Version")
    ),
    responses(
        (status = 200, description = "List matching workers", body = Vec<Worker>)
    )
)]
pub async fn discover_by_capability_and_version(
    State(state): State<Arc<DiscoveryApiState>>,
    Path((capability_name, capability_version)): Path<(String, String)>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let workers = state
        .service
        .find_workers_by_capability(&capability_name, Some(&capability_version))
        .map_err(map_err)?;
    Ok(Json(workers))
}
