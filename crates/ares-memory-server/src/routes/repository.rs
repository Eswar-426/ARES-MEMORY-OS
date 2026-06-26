use ares_core::ProjectId;
use ares_query::services::{HealthQueryService, RepositoryQueryService};
use axum::Json;
use serde_json::{json, Value};

/// GET /repository
/// Returns core repository identity and metadata.
/// Delegates entirely to ares-query.
pub async fn get_repository() -> Json<Value> {
    // Use a placeholder project_id; in R1.5C this will come from shared AppState.
    let project_id = ProjectId::from("default");
    let result = RepositoryQueryService::execute(&project_id);
    Json(json!(result))
}

/// GET /repository/health
/// Returns aggregated health scores across all intelligence dimensions.
/// Delegates entirely to ares-query.
pub async fn get_repository_health() -> Json<Value> {
    let project_id = ProjectId::from("default");
    let result = HealthQueryService::execute(&project_id);
    Json(json!(result))
}
