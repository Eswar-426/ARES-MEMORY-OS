use ares_app::AppState;
use ares_core::ProjectId;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ReindexRequest {
    pub project_id: String,
}

#[derive(Serialize, ToSchema)]
pub struct ReindexResponse {
    pub memories_reindexed: u32,
}

#[utoipa::path(
    post,
    path = "/api/v1/memory/reindex",
    request_body = ReindexRequest,
    responses(
        (status = 200, description = "Re-index complete", body = ReindexResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn reindex(
    State(state): State<AppState>,
    Json(req): Json<ReindexRequest>,
) -> Result<Json<ReindexResponse>, (axum::http::StatusCode, String)> {
    info!(project_id = %req.project_id, "Reindex API request received");
    let project_id = ProjectId::from(req.project_id);

    let count = state
        .semantic_search
        .reindex_project(&project_id)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update embedding count metric
    if let Ok(total) = state.semantic_search.embedding_count() {
        metrics::gauge!("ares_embedding_count").set(total as f64);
    }

    Ok(Json(ReindexResponse {
        memories_reindexed: count,
    }))
}
