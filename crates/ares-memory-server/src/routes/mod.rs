pub mod query;
pub mod repository;
pub mod system;

use axum::{routing::get, Router};

/// Build the complete ARES HTTP router.
/// Strictly enforces the gateway pattern: no intelligence logic lives here.
pub fn build_router() -> Router {
    Router::new()
        // System / Introspection
        .route("/health", get(system::health))
        .route("/version", get(system::version))
        .route("/capabilities", get(system::capabilities))
        .route("/architecture", get(system::architecture))
        // Repository
        .route("/repository", get(repository::get_repository))
        .route("/repository/health", get(repository::get_repository_health))
        // Query (POST because bodies carry node context)
        .route("/query/why", axum::routing::post(query::query_why))
        .route("/query/lineage", axum::routing::post(query::query_lineage))
        .route("/query/impact", axum::routing::post(query::query_impact))
        .route("/query/owner", axum::routing::post(query::query_owner))
}
