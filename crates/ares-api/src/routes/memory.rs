use ares_app::AppState;
use ares_core::{CreateMemoryInput, Memory, MemorySearchResult, ProjectId};
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct SearchMemoryRequest {
    pub project_id: String,
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Deserialize, ToSchema)]
pub struct StoreMemoryRequest {
    pub project_id: String,
    pub memory_type: String,
    pub title: String,
    pub content: serde_json::Value,
    pub importance: Option<String>,
    pub source: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/memory/search",
    request_body = SearchMemoryRequest,
    responses((status = 200, description = "Search results", body = Vec<MemorySearchResult>))
)]
pub async fn search_memory(
    State(state): State<AppState>,
    Json(req): Json<SearchMemoryRequest>,
) -> Json<Vec<MemorySearchResult>> {
    let project_id = ProjectId::from(req.project_id);
    let limit = req.limit.unwrap_or(20);

    match state.memory_repo.search(&project_id, &req.query, limit) {
        Ok(results) => Json(results),
        Err(_) => Json(vec![]),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/memory/create",
    request_body = CreateMemoryInput,
    responses((status = 200, description = "Created memory", body = Memory))
)]
pub async fn create_memory(
    State(state): State<AppState>,
    Json(req): Json<CreateMemoryInput>,
) -> Result<Json<Memory>, axum::http::StatusCode> {
    match state.memory_repo.create(req) {
        Ok(memory) => Ok(Json(memory)),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}

/// Store a memory with simplified input (productization endpoint).
#[utoipa::path(
    post,
    path = "/api/v1/memory/store",
    request_body = StoreMemoryRequest,
    responses((status = 200, description = "Stored memory", body = Memory))
)]
pub async fn store_memory(
    State(state): State<AppState>,
    Json(req): Json<StoreMemoryRequest>,
) -> Result<Json<Memory>, axum::http::StatusCode> {
    let memory_type = req
        .memory_type
        .parse()
        .unwrap_or(ares_core::MemoryType::Feature);
    let importance = req.importance.and_then(|s| s.parse().ok());
    let source = req.source.and_then(|s| s.parse().ok());

    let input = CreateMemoryInput {
        project_id: ProjectId::from(req.project_id),
        memory_type,
        title: req.title,
        content: req.content,
        confidence: Some(1.0),
        importance,
        source,
        ai_assisted: Some(false),
    };

    match state.memory_repo.create(input) {
        Ok(memory) => Ok(Json(memory)),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}
