use crate::models::TimelinePageResponse;
use crate::models::GraphNodePageResponse;
use ares_decision_intelligence::integration::DecisionCoverage;
use ares_decision_intelligence::integration::DecisionSummary;
use crate::models::DecisionPageResponse;
use ares_requirements::integration::RequirementSummary;
use ares_core::types::node::ImpactGraph;
use ares_requirements::integration::RequirementCoverage;
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
pub mod models;

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
        routes::facade_memory::why,
        routes::facade_memory::who,
        routes::facade_memory::impact,
        routes::facade_memory::evolution,
        routes::facade_memory::context,
        routes::facade_health::certification,
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
        // Extractor
        routes::extractor::extract_knowledge,
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
            ares_project_memory::types::ArchitectureStyle,
            ares_memory_evolution::models::ChangeType,
            ares_project_memory::types::ComponentInfo,
            ares_project_memory::types::DependencyType,
            ares_core::types::node::EdgeType,
            ares_core::types::node::NodeType,
            ares_requirements::models::RequirementPriority,
            ares_decision_dna::models::requirement::RequirementStatus,
            ares_requirements::models::RequirementType,
            ares_context_generator::types::SectionPriority,

            ares_core::id::DecisionId,
            ares_core::id::EventId,
            ares_core::id::MemoryId,
            ares_core::id::NodeId,
            ares_core::id::ProjectId,
            ares_core::id::WorkflowId,

            ares_core::types::decision::Alternative,
            ares_project_memory::types::ApiEndpoint,
            ares_project_memory::types::ArchitectureProfile,
            ares_project_memory::types::BugSummary,
            ares_project_memory::types::ChangeRecord,
            ares_context_generator::types::ContextSection,
            ares_core::types::decision::DecisionStatus,
            ares_project_memory::types::DependencyInfo,
            ares_project_memory::types::FeatureSummary,
            ares_project_memory::types::FolderTree,
            ares_core::types::decision::FutureImpact,
            ares_core::types::node::GraphEdge,
            ares_core::types::node::GraphNode,
            ares_core::types::node::ImpactEntry,
            ares_core::types::memory::ImportanceLevel,
            ares_project_memory::types::LanguageProfile,
            ares_core::types::memory::MemorySource,
            ares_core::types::memory::MemoryStatus,
            ares_coordination::distributed::node::NodeId,
            ares_project_memory::types::ProjectStats,
            ares_core::types::decision::ReasoningStep,
            ares_core::types::decision::Risk,
            crate::routes::semantic::SemanticSearchResultDto,
            ares_core::types::plan::TaskStatus,
            DecisionCoverage,
            DecisionSummary,
            RequirementCoverage,
            RequirementSummary,
            ares_orchestrator::runtime::dlq::models::DeadLetterItem,
            ares_orchestrator::runtime::execution::models::DistributedExecution,
            ares_orchestrator::runtime::queue::models::WorkflowQueueItem,
            DecisionPageResponse,
            GraphNodePageResponse,
            TimelinePageResponse,

            ares_context_injector::types::ContextPackage,
            ares_agent::services::context_builder::ContextSnapshot,
            ares_core::types::intelligence::ContradictionRecord,
            ares_core::types::memory::CreateMemoryInput,
            crate::routes::projects::CreateProjectRequest,
            ares_core::types::decision::Decision,
            crate::routes::decisions::DecisionHistoryRequest,
            crate::routes::contradictions::DetectContradictionsRequest,
            crate::routes::snapshot::GenerateContextRequest,
            crate::routes::snapshot::GenerateSnapshotRequest,
            crate::routes::context::GetContextRequest,
            ares_core::types::plan::Goal,
            crate::routes::snapshot::ImportSnapshotRequest,
            crate::routes::context::InjectContextRequest,
            ares_core::types::memory::Memory,
            ares_core::types::memory::MemorySearchResult,
            ares_core::types::plan::Milestone,
            ares_core::types::plan::PlanStatus,
            ares_context_generator::types::PortableContext,
            ares_core::types::project::Project,
            crate::routes::projects::ProjectListResponse,
            ares_project_memory::types::ProjectSnapshot,
            crate::routes::reindex::ReindexRequest,
            crate::routes::reindex::ReindexResponse,
            crate::routes::scan::ScanResult,
            crate::routes::memory::SearchMemoryRequest,
            crate::routes::semantic::SemanticSearchRequest,
            crate::routes::semantic::SemanticSearchResponseDto,
            ares_project_memory::snapshot::SnapshotMeta,
            crate::routes::snapshot::SnapshotResponse,
            crate::routes::memory::StoreMemoryRequest,
            ares_core::types::plan::Task,
            ares_core::types::plan::TaskDependency,
            ImpactGraph,
            ares_core::types::project::ProjectMaturity,
            ares_core::types::memory::MemoryType,

            crate::models::ApiErrorDetail,
            crate::models::ApiErrorEnvelope,
            crate::models::HealthStatus,
            crate::models::EvolutionResult,
            crate::models::MemoryContextPackage,
            // ApiResponse Concrete Aliases
            crate::models::ApiResponseHealthStatus,
            crate::models::ApiResponseValue,
            crate::models::ApiResponseEvolutionResult,
            crate::models::ApiResponseMemoryContextPackage,
            crate::models::ApiResponseCertification,
            // Pagination Concrete DTOs
            GraphNodePageResponse,
            TimelinePageResponse,
            DecisionPageResponse,
            // Workflow API Models
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
            // Extractor
            routes::extractor::ExtractKnowledgeRequest,
        )
    ),
    tags(
        (name = "ares", description = "ARES MemoryOS API")
    ),
    modifiers(&SecurityAddon),
    security(
        ("BearerAuth" = []),
        ("ApiKeyAuth" = [])
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "BearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
            components.add_security_scheme(
                "ApiKeyAuth",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-API-Key"),
                    ),
                ),
            );
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    routes::observability::init_metrics();

    let assembler = ares_memory_intelligence::assembler::MemoryContextAssembler::default_from_store(state.store.clone());
    let memory_facade = std::sync::Arc::new(ares_memory_intelligence::facade::MemoryFacade::new(std::sync::Arc::new(assembler)));
    let validation_runner = std::sync::Arc::new(ares_validation::validation_runner::ValidationRunner::new(
        std::sync::Arc::new(state.store.clone()),
        std::sync::Arc::new(ares_memory_intelligence::assembler::MemoryContextAssembler::default_from_store(state.store.clone()))
    ));

    let facade_router = Router::new()
        .route("/memory/why/:id", get(routes::facade_memory::why))
        .route("/memory/who/:id", get(routes::facade_memory::who))
        .route("/memory/impact/:id", get(routes::facade_memory::impact))
        .route("/memory/evolution/:id", get(routes::facade_memory::evolution))
        .route("/memory/facade_context/:id", get(routes::facade_memory::context))
        .with_state(memory_facade);

    let cert_router = Router::new()
        .route("/memory/certification", get(routes::facade_health::certification))
        .with_state(validation_runner);

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
        // Extractor routes
        .route("/knowledge/extract", post(routes::extractor::extract_knowledge))
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
        .route("/api/v1/health", get(routes::observability::health))
        .route("/metrics", get(routes::observability::metrics))
        .route("/api/v1/metrics", get(routes::observability::metrics))
        .route(
            "/api/v1/telemetry/latest",
            get(routes::telemetry::get_latest_telemetry),
        )
        .nest("/api/v1", api_routes)
        .nest("/api/v1", admin_routes)
        .nest("/api/v1/orchestrator", combined_orchestrator_routes)
        .nest("/api/v1", facade_router)
        .nest("/api/v1", cert_router)
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
