use ares_app::AppState;
use ares_core::AresError;
use ares_planner::planner::{MockPlannerProvider, PlannerEngine};
use ares_store::SqlitePlanRepository;
use axum::{
    extract::{Path, State},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreatePlanRequest {
    pub goal: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct PlanGraphResponse {
    pub nodes: Vec<PlanGraphNode>,
    pub edges: Vec<PlanGraphEdge>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct PlanGraphNode {
    pub id: String,
    pub label: String,
    pub status: String,
    pub complexity: String,
    pub duration: Option<i32>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct PlanGraphEdge {
    pub from: String,
    pub to: String,
}

/// Create a new plan from a goal
#[utoipa::path(
    post,
    path = "/api/v1/plans/create",
    request_body = CreatePlanRequest,
    responses(
        (status = 200, description = "Plan created successfully", body = PlanDetails),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn create_plan(
    State(state): State<AppState>,
    Json(req): Json<CreatePlanRequest>,
) -> Response {
    let provider = Box::new(MockPlannerProvider);
    let engine = PlannerEngine::new(state.store.clone(), provider);
    let res = engine.create_plan_from_goal(&req.goal, &req.priority).await;
    super::into_response(res)
}

/// List all generated plans
#[utoipa::path(
    get,
    path = "/api/v1/plans",
    responses(
        (status = 200, description = "List of plans retrieved", body = [Plan]),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn list_plans(State(state): State<AppState>) -> Response {
    let repo = SqlitePlanRepository::new(state.store.clone());
    let res = repo.list_plans();
    super::into_response(res)
}

/// Get plan details by ID
#[utoipa::path(
    get,
    path = "/api/v1/plans/{id}",
    responses(
        (status = 200, description = "Plan details retrieved", body = PlanDetails),
        (status = 404, description = "Plan not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn get_plan(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let repo = SqlitePlanRepository::new(state.store.clone());
    let res = repo
        .get_plan_details(&id)
        .and_then(|opt| opt.ok_or_else(|| AresError::not_found("plan", &id)));
    super::into_response(res)
}

/// Get task dependency graph for a plan
#[utoipa::path(
    get,
    path = "/api/v1/plans/{id}/graph",
    responses(
        (status = 200, description = "Plan task graph retrieved", body = PlanGraphResponse),
        (status = 404, description = "Plan not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn get_plan_graph(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let repo = SqlitePlanRepository::new(state.store.clone());
    let res = repo.get_plan_details(&id).and_then(|opt| {
        let details = opt.ok_or_else(|| AresError::not_found("plan", &id))?;

        let nodes = details
            .tasks
            .iter()
            .map(|t| PlanGraphNode {
                id: t.id.clone(),
                label: t.title.clone(),
                status: t.status.to_string(),
                complexity: t.complexity.clone().unwrap_or_else(|| "Medium".to_string()),
                duration: t.estimated_duration,
            })
            .collect();

        let edges = details
            .dependencies
            .iter()
            .map(|d| PlanGraphEdge {
                from: d.depends_on_id.clone(),
                to: d.task_id.clone(),
            })
            .collect();

        Ok(PlanGraphResponse { nodes, edges })
    });
    super::into_response(res)
}
