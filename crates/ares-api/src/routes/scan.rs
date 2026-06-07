use ares_app::AppState;
use axum::{extract::State, Json};
use utoipa::ToSchema;
use serde::Serialize;

#[derive(Serialize, ToSchema)]
pub struct ScanResult {
    pub success: bool,
    pub new_memories: usize,
}

#[utoipa::path(
    post,
    path = "/api/v1/scan",
    responses((status = 200, description = "Trigger project scan", body = ScanResult))
)]
pub async fn scan_project(State(_state): State<AppState>) -> Json<ScanResult> {
    // Week 6 feature: Call Scanner from AppState
    Json(ScanResult {
        success: true,
        new_memories: 0,
    })
}
