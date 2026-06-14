use ares_app::AppState;
use ares_core::ExtractionConfig;
use ares_extractor::{ExtractionEngine, MockExtractorProvider};
use axum::{extract::State, response::Response, Json};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ExtractKnowledgeRequest {
    /// Git commit hash. Defaults to HEAD if omitted.
    pub commit_hash: Option<String>,
    /// Path to the repository root. Defaults to current directory if omitted.
    pub repo_path: Option<String>,
    /// Project ID to associate the extracted knowledge with.
    pub project_id: Option<String>,
    /// Confidence threshold override. Default: 0.80
    pub confidence_threshold: Option<f32>,
}

/// Extract knowledge from a git commit
#[utoipa::path(
    post,
    path = "/api/v1/knowledge/extract",
    request_body = ExtractKnowledgeRequest,
    responses(
        (status = 200, description = "Knowledge extracted successfully"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn extract_knowledge(
    State(state): State<AppState>,
    Json(req): Json<ExtractKnowledgeRequest>,
) -> Response {
    let repo_path = req
        .repo_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut config = ExtractionConfig::default();
    if let Some(threshold) = req.confidence_threshold {
        config.confidence_threshold = threshold;
    }

    let provider = Box::new(MockExtractorProvider);
    let engine = ExtractionEngine::new(state.store.clone(), provider, config);

    let result = engine
        .extract_from_commit(
            &repo_path,
            req.commit_hash.as_deref(),
            req.project_id.as_deref(),
        )
        .await;

    super::into_response(result)
}
