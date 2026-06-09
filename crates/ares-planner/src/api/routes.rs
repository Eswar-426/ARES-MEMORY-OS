use crate::coordinator::planner_coordinator::PlannerCoordinator;
use crate::models::goal::Goal;
use ares_core::id::PlanId;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct PlannerAppState {
    pub coordinator: Arc<PlannerCoordinator>,
}

pub fn planner_routes() -> Router<PlannerAppState> {
    Router::new()
        .route("/goals", post(submit_goal))
        .route("/plans/:id", get(get_plan))
}

/// POST /goals
async fn submit_goal(
    State(state): State<PlannerAppState>,
    Json(goal): Json<Goal>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.coordinator.generate_plan(&goal) {
        Ok(plan_id) => Ok(Json(json!({ "status": "success", "plan_id": plan_id }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

/// GET /plans/:id
async fn get_plan(
    Path(_id): Path<PlanId>,
    State(_state): State<PlannerAppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Real implementation would fetch from PlannerRepository
    Ok(Json(json!({ "status": "pending" })))
}
