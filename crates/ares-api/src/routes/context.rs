use ares_agent::services::context_builder::{ContextBudget, ContextSnapshot};
use ares_app::AppState;
use ares_context_injector::{ContextInjector, ContextPackage, TokenBudget};
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
        Json(ContextSnapshot {
            memories: vec![],
            decisions: vec![],
            graph_nodes: vec![],
            graph_edges: vec![],
            estimated_tokens: 0,
        })
    }
}

#[derive(Deserialize, ToSchema)]
pub struct InjectContextRequest {
    pub project_id: String,
    pub prompt: String,
    pub budget: Option<String>, // "small", "medium", "large", "maximum"
}

#[utoipa::path(
    post,
    path = "/api/v1/context/inject",
    request_body = InjectContextRequest,
    responses((status = 200, description = "AI-ready context injected into prompt", body = ContextPackage))
)]
pub async fn inject_context(
    State(state): State<AppState>,
    Json(req): Json<InjectContextRequest>,
) -> Result<Json<ContextPackage>, (axum::http::StatusCode, String)> {
    let budget = match req.budget.as_deref() {
        Some("small") => TokenBudget::Small,
        Some("large") => TokenBudget::Large,
        Some("maximum") => TokenBudget::Maximum,
        _ => TokenBudget::Medium,
    };

    let injector = ContextInjector::new(state.store.clone());
    match injector.inject(&req.project_id, &req.prompt, budget).await {
        Ok(package) => Ok(Json(package)),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
