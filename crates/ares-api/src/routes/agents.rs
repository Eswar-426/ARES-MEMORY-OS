use crate::routes::into_response;
use ares_app::AppState;
use ares_core::{AgentHealth, AgentId, AgentInfo};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

/// Register a new agent
#[utoipa::path(
    post,
    path = "/api/v1/agents/register",
    request_body = AgentInfo,
    responses(
        (status = 200, description = "Agent registered successfully")
    )
)]
pub async fn register_agent(
    State(state): State<AppState>,
    Json(payload): Json<AgentInfo>,
) -> impl IntoResponse {
    let result = state.agent_registry.register(payload);
    if result.is_ok() {
        metrics::counter!("agent_registrations_total").increment(1);
    }
    into_response(result)
}

/// List all agents
#[utoipa::path(
    get,
    path = "/api/v1/agents",
    responses(
        (status = 200, description = "List of agents", body = Vec<AgentInfo>)
    )
)]
pub async fn list_agents(State(state): State<AppState>) -> impl IntoResponse {
    metrics::counter!("agent_list_requests_total").increment(1);
    let result = state.workflow_repo.list_agents();
    into_response::<Vec<AgentInfo>>(result)
}

/// Agent heartbeat endpoint
#[utoipa::path(
    post,
    path = "/api/v1/agents/{id}/heartbeat",
    request_body = AgentHealth,
    params(
        ("id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Heartbeat received")
    )
)]
pub async fn agent_heartbeat(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(health): Json<AgentHealth>,
) -> impl IntoResponse {
    let agent_id = AgentId(id);
    let result = state.agent_registry.health_check(&agent_id, health);
    if result.is_ok() {
        metrics::counter!("agent_heartbeat_total").increment(1);
    }
    into_response(result)
}
