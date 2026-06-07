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
    ),
    components(
        schemas()
    ),
    tags(
        (name = "ares", description = "ARES MemoryOS API")
    )
)]
pub struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
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
        .layer(axum::middleware::from_fn(auth::auth_middleware));

    Router::new()
        .route("/", get(root))
        .route("/health", get(routes::observability::health))
        .route("/metrics", get(routes::observability::metrics))
        .nest("/api/v1", api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn root() -> &'static str {
    "ARES API Platform Running"
}
