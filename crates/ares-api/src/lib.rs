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
        routes::memory::store_memory,
        routes::memory::get_memory_graph,
        routes::memory::get_memory_timeline,
        routes::memory::get_memory_decisions,
        routes::memory::get_memory_context,
        routes::context::get_context,
        routes::context::inject_context,
        routes::decisions::decision_history,
        routes::contradictions::detect_contradictions,
        routes::semantic::semantic_search,
        routes::reindex::reindex,
        // Snapshots & Context
        routes::snapshot::generate_snapshot,
        routes::snapshot::get_snapshot,
        routes::snapshot::export_snapshot,
        routes::snapshot::import_snapshot,
        routes::snapshot::get_project_context,
        routes::snapshot::generate_context,
        routes::snapshot::list_snapshots,
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
        // Planner
        routes::planner::create_plan,
        routes::planner::list_plans,
        routes::planner::get_plan,
        routes::planner::get_plan_graph,
        // Orchestrator
        ares_orchestrator::control::workers::api::register_worker,
        ares_orchestrator::control::heartbeat::api::worker_heartbeat,
        ares_orchestrator::control::discovery::api::discover_by_capability,
        ares_orchestrator::control::discovery::api::discover_by_capability_and_version,
        ares_orchestrator::control::health::api::get_worker_health,
        ares_orchestrator::control::analytics::api::get_analytics,
        ares_orchestrator::runtime::queue::api::enqueue_workflow,
        ares_orchestrator::runtime::dlq::api::list_dlq,
        ares_orchestrator::runtime::execution::api::list_distributed_executions,
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
            // Orchestrator models
            ares_orchestrator::control::workers::models::Worker,
            ares_orchestrator::control::workers::models::WorkerResources,
            ares_orchestrator::control::workers::models::WorkerCapability,
            ares_orchestrator::control::workers::models::WorkerStatus,
            ares_orchestrator::control::workers::dto::WorkerRegistrationRequest,
            ares_orchestrator::control::workers::dto::WorkerStatusUpdateRequest,
            ares_orchestrator::control::health::worker_health::WorkerHealth,
            ares_orchestrator::control::analytics::service::OrchestratorAnalytics,
            ares_orchestrator::runtime::queue::models::WorkflowQueueItem,
            ares_orchestrator::runtime::queue::models::QueueStatus,
            ares_orchestrator::runtime::queue::dto::EnqueueRequest,
            ares_orchestrator::runtime::dlq::models::DeadLetterItem,
            ares_orchestrator::runtime::execution::models::DistributedExecution,
            ares_orchestrator::runtime::execution::models::DistributedExecutionAttempt,
            ares_orchestrator::runtime::execution::models::WorkflowExecutionStep,
            ares_orchestrator::runtime::leases::models::JobLease,
            ares_orchestrator::runtime::retry::RetryPolicy,
            // Planner
            ares_core::Plan,
            ares_core::PlanDetails,
            routes::planner::CreatePlanRequest,
            routes::planner::PlanGraphResponse,
            routes::planner::PlanGraphNode,
            routes::planner::PlanGraphEdge,
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
        .route("/memory/store", post(routes::memory::store_memory))
        .route("/memory/graph", get(routes::memory::get_memory_graph))
        .route("/memory/timeline", get(routes::memory::get_memory_timeline))
        .route(
            "/memory/decisions",
            get(routes::memory::get_memory_decisions),
        )
        .route(
            "/memory/context",
            post(routes::snapshot::generate_context).get(routes::memory::get_memory_context),
        )
        .route("/context", post(routes::context::get_context))
        .route("/context/inject", post(routes::context::inject_context))
        // Snapshot routes
        .route(
            "/project/snapshot",
            post(routes::snapshot::generate_snapshot),
        )
        .route("/project/:id/snapshot", get(routes::snapshot::get_snapshot))
        .route("/project/export", post(routes::snapshot::export_snapshot))
        .route("/project/import", post(routes::snapshot::import_snapshot))
        .route(
            "/project/:id/context",
            get(routes::snapshot::get_project_context),
        )
        .route(
            "/project/:id/snapshots",
            get(routes::snapshot::list_snapshots),
        )
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
        .nest("/chat", routes::import::router())
        // Planner routes
        .route("/plans/create", post(routes::planner::create_plan))
        .route("/plans", get(routes::planner::list_plans))
        .route("/plans/:id", get(routes::planner::get_plan))
        .route("/plans/:id/graph", get(routes::planner::get_plan_graph))
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
        .nest("/knowledge", routes::knowledge::router(state.store.clone()))
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

    // Orchestrator initialization
    let orchestrator_config = ares_orchestrator::control::config::OrchestratorConfig::default();
    let orchestrator =
        ares_orchestrator::start_orchestrator(state.store.clone(), orchestrator_config).unwrap();

    let workers_router = Router::new()
        .route(
            "/",
            post(ares_orchestrator::control::workers::api::register_worker),
        )
        .with_state(orchestrator.workers_api_state);

    let heartbeat_router = Router::new()
        .route(
            "/:id/heartbeat",
            post(ares_orchestrator::control::heartbeat::api::worker_heartbeat),
        )
        .with_state(orchestrator.heartbeat_api_state);

    let discovery_router = Router::new()
        .route(
            "/discovery/:capability_name",
            get(ares_orchestrator::control::discovery::api::discover_by_capability),
        )
        .route(
            "/discovery/:capability_name/:capability_version",
            get(ares_orchestrator::control::discovery::api::discover_by_capability_and_version),
        )
        .with_state(orchestrator.discovery_api_state);

    let health_router = Router::new()
        .route(
            "/health",
            get(ares_orchestrator::control::health::api::get_worker_health),
        )
        .with_state(orchestrator.health_api_state);

    let analytics_router = Router::new()
        .route(
            "/analytics",
            get(ares_orchestrator::control::analytics::api::get_analytics),
        )
        .with_state(orchestrator.analytics_api_state);

    let queue_router = Router::new()
        .route(
            "/queue",
            post(ares_orchestrator::runtime::queue::api::enqueue_workflow),
        )
        .with_state(orchestrator.queue_api_state);

    let dlq_router = Router::new()
        .route("/dlq", get(ares_orchestrator::runtime::dlq::api::list_dlq))
        .with_state(orchestrator.dlq_api_state);

    let execution_router = Router::new()
        .route(
            "/executions/distributed",
            get(ares_orchestrator::runtime::execution::api::list_distributed_executions),
        )
        .with_state(orchestrator.execution_api_state);

    let event_store_router = ares_orchestrator::events::store::api::router()
        .with_state(orchestrator.event_store_api_state);

    let ws_router =
        ares_orchestrator::events::websocket::api::router().with_state(orchestrator.ws_api_state);

    let sse_router =
        ares_orchestrator::events::sse::api::router().with_state(orchestrator.sse_api_state);

    let combined_orchestrator_routes = Router::new()
        .nest("/workers", workers_router)
        .nest("/workers", heartbeat_router)
        .nest("/workers", discovery_router)
        .nest("/workers", health_router)
        .nest("/", analytics_router)
        .nest("/", queue_router)
        .nest("/", dlq_router)
        .nest("/", execution_router)
        .nest("/events", event_store_router)
        .nest("/", ws_router)
        .nest("/", sse_router);

    Router::new()
        .route("/", get(root))
        .route("/health", get(routes::observability::health))
        .route("/metrics", get(routes::observability::metrics))
        .route(
            "/api/v1/telemetry/latest",
            get(routes::telemetry::get_latest_telemetry),
        )
        .nest("/api/v1", api_routes)
        .nest("/api/v1", admin_routes)
        .nest("/api/v1/orchestrator", combined_orchestrator_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(tower_http::timeout::TimeoutLayer::new(
            std::time::Duration::from_secs(30),
        ))
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
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

        // Planner
        assert!(paths.paths.contains_key("/api/v1/plans/create"));
        assert!(paths.paths.contains_key("/api/v1/plans"));
        assert!(paths.paths.contains_key("/api/v1/plans/{id}"));
        assert!(paths.paths.contains_key("/api/v1/plans/{id}/graph"));
    }
}
