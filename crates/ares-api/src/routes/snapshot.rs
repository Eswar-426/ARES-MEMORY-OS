//! Snapshot routes — generate, import, export project snapshots and context.

use ares_app::AppState;
use ares_context_generator::types::PortableContext;
use ares_context_generator::ContextGenerator;
use ares_core::ProjectId;
use ares_project_memory::snapshot::SnapshotMeta;
use ares_project_memory::types::ProjectSnapshot;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GenerateSnapshotRequest {
    pub project_id: String,
}

#[derive(Serialize, ToSchema)]
pub struct SnapshotResponse {
    pub snapshot_id: String,
    pub project_id: String,
    pub name: String,
    pub stats: serde_json::Value,
}

#[derive(Deserialize, ToSchema)]
pub struct ImportSnapshotRequest {
    pub snapshot_json: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GenerateContextRequest {
    pub project_id: String,
    pub max_tokens: Option<usize>,
}

/// POST /project/snapshot — Generate a new project snapshot.
#[utoipa::path(
    post,
    path = "/api/v1/project/snapshot",
    request_body = GenerateSnapshotRequest,
    responses((status = 200, description = "Snapshot generated", body = SnapshotResponse))
)]
pub async fn generate_snapshot(
    State(state): State<AppState>,
    Json(req): Json<GenerateSnapshotRequest>,
) -> Result<Json<SnapshotResponse>, axum::http::StatusCode> {
    let project_id = ProjectId::from(req.project_id);

    let snapshot = state
        .memory_builder
        .build_snapshot_by_id(&project_id)
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    let snapshot_id = state
        .snapshot_store
        .save(&snapshot)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats = serde_json::to_value(&snapshot.stats).unwrap_or_default();

    Ok(Json(SnapshotResponse {
        snapshot_id,
        project_id: snapshot.project_id,
        name: snapshot.name,
        stats,
    }))
}

/// POST /project/export — Export a project snapshot as JSON.
#[utoipa::path(
    post,
    path = "/api/v1/project/export",
    request_body = GenerateSnapshotRequest,
    responses((status = 200, description = "Snapshot JSON", body = ProjectSnapshot))
)]
pub async fn export_snapshot(
    State(state): State<AppState>,
    Json(req): Json<GenerateSnapshotRequest>,
) -> Result<Json<ProjectSnapshot>, axum::http::StatusCode> {
    let project_id = ProjectId::from(req.project_id);

    // Try to get latest snapshot, or build a new one
    let snapshot = match state.snapshot_store.get_latest(project_id.as_str()) {
        Ok(Some(s)) => s,
        _ => state
            .memory_builder
            .build_snapshot_by_id(&project_id)
            .map_err(|_| axum::http::StatusCode::NOT_FOUND)?,
    };

    Ok(Json(snapshot))
}

/// POST /project/import — Import a project snapshot from JSON.
#[utoipa::path(
    post,
    path = "/api/v1/project/import",
    request_body = ImportSnapshotRequest,
    responses((status = 200, description = "Import result", body = SnapshotResponse))
)]
pub async fn import_snapshot(
    State(state): State<AppState>,
    Json(req): Json<ImportSnapshotRequest>,
) -> Result<Json<SnapshotResponse>, axum::http::StatusCode> {
    let snapshot = ares_project_memory::SnapshotStore::import_json(&req.snapshot_json)
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    let snapshot_id = state
        .snapshot_store
        .save(&snapshot)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats = serde_json::to_value(&snapshot.stats).unwrap_or_default();

    Ok(Json(SnapshotResponse {
        snapshot_id,
        project_id: snapshot.project_id,
        name: snapshot.name,
        stats,
    }))
}

/// GET /project/{id}/context — Generate portable AI context for a project.
#[utoipa::path(
    get,
    path = "/api/v1/project/{id}/context",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses((status = 200, description = "Portable AI context", body = PortableContext))
)]
pub async fn get_project_context(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PortableContext>, axum::http::StatusCode> {
    let project_id = ProjectId::from(id.clone());

    // Try to get latest snapshot, or build a new one
    let snapshot = match state.snapshot_store.get_latest(&id) {
        Ok(Some(s)) => s,
        _ => state
            .memory_builder
            .build_snapshot_by_id(&project_id)
            .map_err(|_| axum::http::StatusCode::NOT_FOUND)?,
    };

    let context = ContextGenerator::generate(&snapshot);
    Ok(Json(context))
}

/// POST /memory/context — Generate portable context (with budget).
#[utoipa::path(
    post,
    path = "/api/v1/memory/context",
    request_body = GenerateContextRequest,
    responses((status = 200, description = "Portable AI context with budget", body = PortableContext))
)]
pub async fn generate_context(
    State(state): State<AppState>,
    Json(req): Json<GenerateContextRequest>,
) -> Result<Json<PortableContext>, axum::http::StatusCode> {
    let project_id = ProjectId::from(req.project_id.clone());

    let snapshot = match state.snapshot_store.get_latest(&req.project_id) {
        Ok(Some(s)) => s,
        _ => state
            .memory_builder
            .build_snapshot_by_id(&project_id)
            .map_err(|_| axum::http::StatusCode::NOT_FOUND)?,
    };

    let context = match req.max_tokens {
        Some(budget) => ContextGenerator::generate_for_budget(&snapshot, budget),
        None => ContextGenerator::generate(&snapshot),
    };

    Ok(Json(context))
}

/// GET /project/{id}/snapshots — List snapshot history.
#[utoipa::path(
    get,
    path = "/api/v1/project/{id}/snapshots",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses((status = 200, description = "Snapshot list", body = Vec<SnapshotMeta>))
)]
pub async fn list_snapshots(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Vec<SnapshotMeta>> {
    match state.snapshot_store.list(&id) {
        Ok(list) => Json(list),
        Err(_) => Json(vec![]),
    }
}

/// GET /project/{id}/snapshot — Get the latest project snapshot as JSON.
#[utoipa::path(
    get,
    path = "/api/v1/project/{id}/snapshot",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses((status = 200, description = "Snapshot JSON", body = ProjectSnapshot))
)]
pub async fn get_snapshot(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProjectSnapshot>, axum::http::StatusCode> {
    let project_id = ProjectId::from(id.clone());

    // Try to get latest snapshot, or build a new one
    let snapshot = match state.snapshot_store.get_latest(&id) {
        Ok(Some(s)) => s,
        _ => state
            .memory_builder
            .build_snapshot_by_id(&project_id)
            .map_err(|_| axum::http::StatusCode::NOT_FOUND)?,
    };

    Ok(Json(snapshot))
}
