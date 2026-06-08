use ares_agent::services::context_builder::{ContextBudget, ReasoningContext};
use ares_app::AppState;
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
    responses((status = 200, description = "AI-ready context", body = ReasoningContext))
)]
pub async fn get_context(
    State(state): State<AppState>,
    Json(req): Json<GetContextRequest>,
) -> Json<ReasoningContext> {
    let project = Project {
        id: ares_core::ProjectId(ares_core::id::new_id()),
        name: "Mock Project".into(),
        description: "Mock Description".into(),
        root_path: state.config.project_path.clone(),
        primary_language: ares_core::Language::Rust.as_str().into(),
        domain: "Mock Domain".into(),
        maturity: ares_core::ProjectMaturity::Greenfield,
        created_at: chrono::Utc::now().timestamp_micros(),
        updated_at: chrono::Utc::now().timestamp_micros(),
        deleted_at: None,
    };

    let budget = ContextBudget::default();
    // Use ContextPipeline to assemble context
    if let Ok(snapshot) = state
        .context_pipeline
        .assemble_context(&project, &req.query, budget)
    {
        Json(snapshot)
    } else {
        // Mock fallback if something fails
        Json(ReasoningContext {
            memories: vec![],
            decisions: vec![],
            contradictions: vec![],
            dependencies: vec![],
            timeline: None,
            confidence: 0.0,
            summary: "".into(),
            estimated_tokens: 0,
        })
    }
}
