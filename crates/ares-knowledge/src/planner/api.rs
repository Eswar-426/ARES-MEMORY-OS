use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use super::models::{ExecutionPlan, ExplainPlanResponse};
use super::service::PlannerIntegrationService;

#[derive(Clone)]
pub struct PlannerApiState {
    pub service: Arc<PlannerIntegrationService>,
}

pub fn router<S>(state: PlannerApiState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/goals/:id/dependencies", get(get_dependencies))
        .route("/goals/:id/plan", get(get_plan))
        .route("/goals/:id/explain", get(explain_plan))
        .with_state(state)
}

async fn get_dependencies(
    State(state): State<PlannerApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Uuid>>, String> {
    let deps = state
        .service
        .find_dependencies(id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(deps))
}

async fn get_plan(
    State(state): State<PlannerApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionPlan>, String> {
    let plan = state
        .service
        .find_execution_plan(id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(plan))
}

async fn explain_plan(
    State(state): State<PlannerApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExplainPlanResponse>, String> {
    let explanation = state
        .service
        .explain_plan(id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(explanation))
}
