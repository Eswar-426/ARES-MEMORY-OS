use ares_app::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod auth;
pub mod routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::observability::health,
        routes::observability::metrics,
        routes::projects::list_projects,
        routes::projects::create_project,
        routes::projects::get_project,
        routes::scan::scan_project,
        routes::memory::search_memory,
        routes::memory::create_memory,
        routes::context::get_context,
        routes::decisions::decision_history,
        routes::contradictions::detect_contradictions,
        routes::semantic::semantic_search,
        routes::reindex::reindex,
        // Workflows
        routes::workflows::run_workflow,
        routes::workflows::pause_workflow,
        routes::workflows::resume_workflow,
        routes::workflows::cancel_workflow,
        routes::workflows::search_executions,
        routes::workflows::replay_workflow,
        routes::workflows::get_analytics,
        routes::workflows::visualize_workflow,
        // Agents
        routes::agents::register_agent,
        routes::agents::list_agents,
        routes::agents::agent_heartbeat,
    ),
    components(
        schemas(
            ares_core::types::workflow_api::PageRequest,
            ares_core::types::workflow_api::PageResponseExecutionSummary,
            ares_core::types::workflow_api::WorkflowRunRequest,
            ares_core::types::workflow_api::WorkflowSummary,
            ares_core::types::workflow_api::WorkflowVersionSummary,
            ares_core::types::workflow_api::ExecutionSearchRequest,
            ares_core::types::workflow_api::ExecutionSummary,
            ares_core::types::workflow_api::ExecutionDetails,
            ares_core::types::workflow_api::ExecutionVersion,
            ares_core::types::workflow_api::ReplayVerification,
            ares_core::types::workflow_api::ReplayReport,
            ares_core::types::workflow_api::ReplayAuditEntry,
            ares_core::types::workflow_api::WorkflowAnalyticsReport,
            ares_core::types::workflow_api::AgentSummary,
            ares_core::types::workflow_api::AgentPerformanceReport,
            ares_core::types::workflow_api::WorkflowGraphResponse,
            ares_core::types::workflow_api::WorkflowTimelineEvent,
            ares_core::AgentInfo,
            ares_core::AgentHealth,
            ares_core::AgentId,
            ares_core::AgentPerformance,
            ares_core::ExecutionId,
            ares_core::WorkflowStatus,
            routes::observability::HealthStatus,
        )
    ),
    tags(
        (name = "ares", description = "ARES MemoryOS API")
    )
)]
pub struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    routes::observability::init_metrics();

    let api_routes = Router::new()
        .route(
            "/projects",
            get(routes::projects::list_projects).post(routes::projects::create_project),
        )
        .route("/projects/:id", get(routes::projects::get_project))
        .route("/scan", post(routes::scan::scan_project))
        .route("/memory/search", post(routes::memory::search_memory))
        .route("/memory/create", post(routes::memory::create_memory))
        .route("/context", post(routes::context::get_context))
        .route(
            "/decisions/history",
            post(routes::decisions::decision_history),
        )
        .route(
            "/contradictions",
            post(routes::contradictions::detect_contradictions),
        )
        .route(
            "/memory/semantic-search",
            post(routes::semantic::semantic_search),
        )
        .route("/memory/reindex", post(routes::reindex::reindex))
        // Agent routes
        .route("/agents", get(routes::agents::list_agents))
        .route("/agents/register", post(routes::agents::register_agent))
        .route(
            "/agents/:id/heartbeat",
            post(routes::agents::agent_heartbeat),
        )
        // Workflow routes
        .route("/workflows/run", post(routes::workflows::run_workflow))
        .route(
            "/workflows/executions/search",
            get(routes::workflows::search_executions),
        )
        .route(
            "/workflows/executions/:id/pause",
            post(routes::workflows::pause_workflow),
        )
        .route(
            "/workflows/executions/:id/resume",
            post(routes::workflows::resume_workflow),
        )
        .route(
            "/workflows/executions/:id/retry",
            post(routes::workflows::retry_workflow),
        )
        .route(
            "/workflows/executions/:id/stream",
            get(routes::workflows::execution_stream),
        )
        .route(
            "/workflows/analytics",
            get(routes::workflows::get_analytics),
        )
        .route(
            "/workflows/visualize/:id",
            get(routes::workflows::visualize_workflow),
        )
        .layer(axum::middleware::from_fn(auth::auth_middleware));

    let admin_routes = Router::new()
        .route(
            "/workflows/executions/:id/cancel",
            post(routes::workflows::cancel_workflow),
        )
        .route(
            "/workflows/replay/:id",
            post(routes::workflows::replay_workflow),
        )
        .layer(axum::middleware::from_fn(auth::admin_middleware));

    Router::new()
        .route("/", get(root))
        .route("/health", get(routes::observability::health))
        .route("/metrics", get(routes::observability::metrics))
        .nest("/api/v1", api_routes)
        .nest("/api/v1", admin_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn root() -> &'static str {
    "ARES API Platform Running"
}

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn openapi_has_no_unresolved_refs() {
        let openapi = ApiDoc::openapi();
        let json = openapi
            .to_json()
            .expect("Failed to serialize OpenAPI to JSON");

        // This is a simple heuristic: if a struct fails to resolve, Swagger often throws errors
        // or the JSON generation would have failed. utoipa checks some of this at compile time.
        // We ensure the JSON does not contain empty components block if there are paths.
        assert!(json.contains("\"components\""));
        assert!(json.contains("\"paths\""));
        assert!(json.contains("ExecutionId"));
        assert!(json.contains("WorkflowStatus"));

        // Ensure no placeholder or literal "unresolved" string got embedded (which sometimes happens with certain schema generators)
        assert!(!json.to_lowercase().contains("unresolved reference"));
    }

    #[test]
    fn openapi_routes_registered() {
        let openapi = ApiDoc::openapi();
        let paths = openapi.paths;

        // Workflows
        assert!(paths.paths.contains_key("/api/v1/workflows/run"));
        assert!(paths
            .paths
            .contains_key("/api/v1/workflows/executions/search"));
        assert!(paths.paths.contains_key("/api/v1/workflows/replay/{id}"));
        assert!(paths.paths.contains_key("/api/v1/workflows/analytics"));
        assert!(paths.paths.contains_key("/api/v1/workflows/visualize/{id}"));

        // Agents
        assert!(paths.paths.contains_key("/api/v1/agents"));
        assert!(paths.paths.contains_key("/api/v1/agents/register"));
        assert!(paths.paths.contains_key("/api/v1/agents/{id}/heartbeat"));
    }
}
