use crate::models::DecisionPageResponse;
use crate::models::GraphNodePageResponse;
use crate::models::TimelinePageResponse;
use ares_core::ImpactGraph;
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

#[derive(Deserialize, ToSchema)]
pub struct MemoryGraphQuery {
    pub project_id: String,
    pub node_type: Option<String>,
    pub search: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/memory/graph",
    params(
        ("project_id" = String, Query, description = "Project ID"),
        ("node_type" = Option<String>, Query, description = "Filter by node type"),
        ("search" = Option<String>, Query, description = "Search query"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size")
    ),
    responses((status = 200, description = "Graph nodes", body = GraphNodePageResponse))
)]
pub async fn get_memory_graph(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<MemoryGraphQuery>,
) -> Result<Json<GraphNodePageResponse>, axum::http::StatusCode>
{
    let project_id = ProjectId::from(query.project_id);
    let pagination = ares_core::types::pagination::Pagination {
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(50),
    };
    let node_type = query.node_type.and_then(|t| t.parse().ok());

    match state.graph_repo.list_nodes_paginated(
        &project_id,
        node_type,
        query.search.as_deref(),
        &pagination,
    ) {
        Ok(page) => Ok(Json(GraphNodePageResponse {
            items: page.items,
            page: page.page,
            page_size: page.page_size,
            total: page.total,
        })),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct MemoryTimelineQuery {
    pub project_id: String,
    pub event_types: Option<String>, // comma separated
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/memory/timeline",
    params(
        ("project_id" = String, Query, description = "Project ID"),
        ("event_types" = Option<String>, Query, description = "Comma separated event types"),
        ("since" = Option<i64>, Query, description = "Since timestamp"),
        ("until" = Option<i64>, Query, description = "Until timestamp"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size")
    ),
    responses((status = 200, description = "Timeline events", body = TimelinePageResponse))
)]
pub async fn get_memory_timeline(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<MemoryTimelineQuery>,
) -> Result<Json<TimelinePageResponse>, axum::http::StatusCode> {
    let project_id = ProjectId::from(query.project_id);
    let pagination = ares_core::types::pagination::Pagination {
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(50),
    };

    let mut event_types = None;
    if let Some(et_str) = query.event_types {
        let mut types = vec![];
        for s in et_str.split(',') {
            // Note: we can map string to EventType, wait, EventType doesn't implement FromStr directly.
            // map_event_type is private in event.rs? No we made it public.
            types.push(ares_store::repositories::event::map_event_type(s));
        }
        event_types = Some(types);
    }

    let filter = ares_core::types::event::TimelineFilter {
        event_types,
        since: query.since,
        until: query.until,
    };

    match state
        .timeline_repo
        .list_paginated(&project_id, filter, &pagination)
    {
        Ok(page) => Ok(Json(TimelinePageResponse {
            items: page.items,
            page: page.page,
            page_size: page.page_size,
            total: page.total,
        })),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct MemoryDecisionsQuery {
    pub project_id: String,
    pub status: Option<String>,
    pub file_path: Option<String>,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/memory/decisions",
    params(
        ("project_id" = String, Query, description = "Project ID"),
        ("status" = Option<String>, Query, description = "Decision status"),
        ("file_path" = Option<String>, Query, description = "File path"),
        ("since" = Option<i64>, Query, description = "Since timestamp"),
        ("until" = Option<i64>, Query, description = "Until timestamp"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size")
    ),
    responses((status = 200, description = "Decisions", body = DecisionPageResponse))
)]
pub async fn get_memory_decisions(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<MemoryDecisionsQuery>,
) -> Result<Json<DecisionPageResponse>, axum::http::StatusCode> {
    let project_id = ProjectId::from(query.project_id);
    let pagination = ares_core::types::pagination::Pagination {
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(50),
    };

    let filter = ares_core::DecisionFilter {
        status: query.status.and_then(|s| s.parse().ok()),
        file_path: query.file_path,
        since: query.since,
        until: query.until,
        stale_days: None,
    };

    match state
        .decision_repo
        .list_paginated(&project_id, filter, &pagination)
    {
        Ok(page) => Ok(Json(DecisionPageResponse {
            items: page.items,
            page: page.page,
            page_size: page.page_size,
            total: page.total,
        })),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct MemoryContextQuery {
    pub memory_id: String,
    pub depth: Option<u8>,
}

#[utoipa::path(
    get,
    path = "/api/v1/memory/context",
    params(
        ("memory_id" = String, Query, description = "Memory ID (or Node ID)"),
        ("depth" = Option<u8>, Query, description = "Traversal depth")
    ),
    responses((status = 200, description = "Memory context graph", body = ImpactGraph))
)]
pub async fn get_memory_context(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<MemoryContextQuery>,
) -> Result<Json<ImpactGraph>, axum::http::StatusCode> {
    let node_id = ares_core::NodeId::from(query.memory_id);
    let depth = query.depth.unwrap_or(2);

    match state.graph_repo.traverse_impact(&node_id, depth) {
        Ok(impact) => Ok(Json(impact)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
