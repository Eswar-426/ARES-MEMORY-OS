use ares_app::AppState;
use ares_core::{CreateMemoryInput, Memory, MemorySearchResult};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct SearchMemoryRequest {
    pub query: String,
    pub limit: usize,
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
    // For now, mock memory search
    // In Week 5, this will call state.retrieval_layer.search(...)
    Json(vec![])
}

#[utoipa::path(
    post,
    path = "/api/v1/memory/create",
    request_body = CreateMemoryInput,
    responses((status = 200, description = "Created memory", body = Memory))
)]
pub async fn create_memory(
    State(state): State<AppState>,
    Json(_req): Json<CreateMemoryInput>,
) -> Json<Memory> {
    // Mock memory creation
    let memory = Memory {
        id: ares_core::id::new_id(),
        project_id: ares_core::id::new_id(),
        source: ares_core::MemorySource::UserAssigned,
        memory_type: ares_core::MemoryType::Concept,
        content: "Mock".into(),
        importance: ares_core::ImportanceLevel::High,
        status: ares_core::MemoryStatus::Active,
        version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: None,
        context_tags: vec![],
    };
    Json(memory)
}
