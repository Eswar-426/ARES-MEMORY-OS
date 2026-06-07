use ares_app::AppState;
use ares_core::Decision;
use axum::{extract::State, Json};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct DecisionHistoryRequest {
    pub project_id: String,
    pub decision_id: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/decisions/history",
    request_body = DecisionHistoryRequest,
    responses((status = 200, description = "Decision history", body = Vec<Decision>))
)]
pub async fn decision_history(
    State(state): State<AppState>,
    Json(_req): Json<DecisionHistoryRequest>,
) -> Json<Vec<Decision>> {
    let project_id = ares_core::ProjectId(ares_core::id::new_id()); // Need parsing from req
    let decision_id = ares_core::DecisionId(ares_core::id::new_id()); // Need parsing from req

    if let Ok(history) = state
        .decision_intelligence
        .decision_history(&project_id, &decision_id)
    {
        Json(history)
    } else {
        Json(vec![])
    }
}
