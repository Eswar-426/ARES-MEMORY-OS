use ares_app::AppState;
use ares_agent::services::context_builder::{ContextSnapshot, ContextBudget};
use ares_core::Project;
use axum::{extract::State, Json};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GetContextRequest {
    pub project_id: String,
    pub query: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/context",
    request_body = GetContextRequest,
    responses((status = 200, description = "AI-ready context", body = ContextSnapshot))
)]
pub async fn get_context(
    State(state): State<AppState>,
    Json(req): Json<GetContextRequest>,
) -> Json<ContextSnapshot> {
    let project = Project {
        id: ares_core::id::new_id(), // Mock ID, real implementation needs to fetch from DB
        name: "Mock Project".into(),
        path: state.config.project_path.clone(),
        language: ares_core::Language::Rust,
        maturity: ares_core::ProjectMaturity::Exploratory,
    };
    
    let budget = ContextBudget::default();
    // Use ContextPipeline to assemble context
    if let Ok(snapshot) = state.context_pipeline.assemble_context(&project, &req.query, budget) {
        Json(snapshot)
    } else {
        // Mock fallback if something fails
        Json(ContextSnapshot {
            memories: vec![],
            decisions: vec![],
            graph_nodes: vec![],
            contradictions: vec![],
        })
    }
}
