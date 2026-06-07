use ares_app::AppState;
use ares_core::ContradictionRecord;
use axum::{extract::State, Json};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct DetectContradictionsRequest {
    pub project_id: String,
    pub nodes_to_check: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/contradictions",
    request_body = DetectContradictionsRequest,
    responses((status = 200, description = "Contradictions found", body = Vec<ContradictionRecord>))
)]
pub async fn detect_contradictions(
    State(state): State<AppState>,
    Json(_req): Json<DetectContradictionsRequest>,
) -> Json<Vec<ContradictionRecord>> {
    let project_id = ares_core::id::new_id(); // Need parsing
    
    if let Ok(contradictions) = state.contradiction_detector.detect_contradictions(&project_id, &[]) {
        Json(contradictions)
    } else {
        Json(vec![])
    }
}
