use ares_agent::services::context_builder::ContextBudget;
use ares_agent::services::dependency_analysis::DependencyAnalysis;
use ares_agent::services::evolution_engine::EvolutionAnalysis;
use ares_agent::services::reasoning_pipeline::ReasoningOutput;
use ares_app::AppState;
use ares_core::Project;
use axum::{extract::State, Json};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ReasonRequest {
    pub query: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/reason",
    request_body = ReasonRequest,
    responses((status = 200, description = "Reasoning context", body = ReasoningOutput))
)]
pub async fn reason(
    State(state): State<AppState>,
    Json(req): Json<ReasonRequest>,
) -> Json<Option<ReasoningOutput>> {
    let project = Project {
        id: ares_core::ProjectId::new(),
        name: "".into(),
        description: "".into(),
        root_path: "".into(),
        primary_language: "".into(),
        domain: "".into(),
        maturity: ares_core::ProjectMaturity::Greenfield,
        created_at: 0,
        updated_at: 0,
        deleted_at: None,
    };
    if let Ok(output) =
        state
            .reasoning_pipeline
            .reason(&project, &req.query, ContextBudget::budget_8k())
    {
        Json(Some(output))
    } else {
        Json(None)
    }
}

#[derive(Deserialize, ToSchema)]
pub struct ImpactAnalysisRequest {
    pub node_id: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/impact-analysis",
    request_body = ImpactAnalysisRequest,
    responses((status = 200, description = "Impact analysis", body = DependencyAnalysis))
)]
pub async fn impact_analysis(
    State(state): State<AppState>,
    Json(req): Json<ImpactAnalysisRequest>,
) -> Json<Option<DependencyAnalysis>> {
    let node_id = ares_core::NodeId::from(req.node_id);
    if let Ok(analysis) = state.dependency_analyzer.impacts(&node_id, Some(3)) {
        Json(Some(analysis))
    } else {
        Json(None)
    }
}

#[derive(Deserialize, ToSchema)]
pub struct TimelineRequest {
    pub memory_id: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/timeline",
    request_body = TimelineRequest,
    responses((status = 200, description = "Timeline", body = EvolutionAnalysis))
)]
pub async fn timeline(
    State(state): State<AppState>,
    Json(req): Json<TimelineRequest>,
) -> Json<Option<EvolutionAnalysis>> {
    if let Ok(timeline) = state.evolution_engine.memory_timeline(&req.memory_id) {
        Json(Some(timeline))
    } else {
        Json(None)
    }
}

#[derive(Deserialize, ToSchema)]
pub struct ExplainDecisionRequest {
    pub decision_id: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/decision/explain",
    request_body = ExplainDecisionRequest,
    responses((status = 200, description = "Decision explanation"))
)]
pub async fn explain_decision(
    State(state): State<AppState>,
    Json(req): Json<ExplainDecisionRequest>,
) -> Json<Option<EvolutionAnalysis>> {
    if let Ok(explanation) = state.evolution_engine.decision_timeline(&req.decision_id) {
        Json(Some(explanation))
    } else {
        Json(None)
    }
}
