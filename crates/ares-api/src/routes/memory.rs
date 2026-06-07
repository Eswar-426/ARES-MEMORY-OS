use ares_app::AppState;
use ares_core::{CreateMemoryInput, Memory, MemorySearchResult};
use axum::{extract::State, Json};
use serde::Deserialize;
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
    State(_state): State<AppState>,
    Json(_req): Json<SearchMemoryRequest>,
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
    State(_state): State<AppState>,
    Json(_req): Json<CreateMemoryInput>,
) -> Json<Memory> {
    // Mock memory creation
    let memory = Memory {
        id: ares_core::MemoryId(ares_core::id::new_id()),
        project_id: ares_core::ProjectId(ares_core::id::new_id()),
        source: ares_core::MemorySource::Human,
        memory_type: ares_core::MemoryType::Feature,
        title: "Mock Title".into(),
        content: "Mock".into(),
        importance: ares_core::ImportanceLevel::High,
        status: ares_core::MemoryStatus::Active,
        version: 1,
        parent_id: None,
        confidence: 1.0,
        ai_assisted: false,
        created_at: chrono::Utc::now().timestamp_micros(),
        updated_at: chrono::Utc::now().timestamp_micros(),
        deleted_at: None,
    };
    Json(memory)
}
