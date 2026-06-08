use ares_agent::services::hybrid_ranking::SemanticSearchResult;
use ares_app::AppState;
use ares_core::{Memory, ProjectId};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct SemanticSearchRequest {
    pub project_id: String,
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Serialize, ToSchema)]
pub struct SemanticSearchResultDto {
    pub memory: Memory,
    pub semantic_score: f32,
    pub keyword_score: f32,
    pub importance_score: f32,
    pub recency_score: f32,
    pub final_score: f32,
}

impl From<SemanticSearchResult> for SemanticSearchResultDto {
    fn from(res: SemanticSearchResult) -> Self {
        Self {
            memory: res.memory,
            semantic_score: res.semantic_score,
            keyword_score: res.keyword_score,
            importance_score: res.importance_score,
            recency_score: res.recency_score,
            final_score: res.final_score,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct SemanticSearchResponseDto {
    pub results: Vec<SemanticSearchResultDto>,
    pub provider: String,
    pub model: String,
    pub total_latency_ms: f64,
}

#[utoipa::path(
    post,
    path = "/api/v1/memory/semantic-search",
    request_body = SemanticSearchRequest,
    responses(
        (status = 200, description = "Semantic search results", body = SemanticSearchResponseDto),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn semantic_search(
    State(state): State<AppState>,
    Json(req): Json<SemanticSearchRequest>,
) -> Result<Json<SemanticSearchResponseDto>, (axum::http::StatusCode, String)> {
    // Record metrics
    metrics::counter!("ares_semantic_search_requests_total").increment(1);

    let project_id = ProjectId::from(req.project_id);
    let limit = req.limit.unwrap_or(10);

    match state
        .semantic_search
        .search(&project_id, &req.query, limit)
        .await
    {
        Ok(response) => {
            metrics::histogram!("ares_semantic_search_latency_seconds")
                .record(response.diagnostics.total_latency_ms / 1000.0);

            Ok(Json(SemanticSearchResponseDto {
                results: response.results.into_iter().map(Into::into).collect(),
                provider: response.diagnostics.provider,
                model: response.diagnostics.model,
                total_latency_ms: response.diagnostics.total_latency_ms,
            }))
        }
        Err(e) => {
            metrics::counter!("ares_semantic_search_failures_total").increment(1);
            Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
