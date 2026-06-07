use ares_app::AppState;
use ares_core::Project;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ProjectListResponse {
    pub projects: Vec<Project>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub path: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    responses((status = 200, description = "List registered projects", body = ProjectListResponse))
)]
pub async fn list_projects(State(state): State<AppState>) -> Json<ProjectListResponse> {
    // For now, return the single configured project
    // Future: Fetch all projects from global db
    let p = Project {
        id: ares_core::ProjectId(ares_core::id::new_id()),
        name: "Mock Project".into(),
        description: "Mock Description".into(),
        root_path: state.config.project_path.clone(),
        primary_language: ares_core::Language::Rust.as_str().into(),
        domain: "Mock Domain".into(),
        maturity: ares_core::ProjectMaturity::Greenfield,
        created_at: chrono::Utc::now().timestamp_micros(),
        updated_at: chrono::Utc::now().timestamp_micros(),
        deleted_at: None,
    };
    Json(ProjectListResponse { projects: vec![p] })
}

#[utoipa::path(
    post,
    path = "/api/v1/projects",
    request_body = CreateProjectRequest,
    responses((status = 200, description = "Project created", body = Project))
)]
pub async fn create_project(
    State(_state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Json<Project> {
    let p = Project {
        id: ares_core::ProjectId(ares_core::id::new_id()),
        name: payload.name,
        description: "Mock Description".into(),
        root_path: payload.path,
        primary_language: ares_core::Language::Rust.as_str().into(),
        domain: "Mock Domain".into(),
        maturity: ares_core::ProjectMaturity::Greenfield,
        created_at: chrono::Utc::now().timestamp_micros(),
        updated_at: chrono::Utc::now().timestamp_micros(),
        deleted_at: None,
    };
    // Future: Insert project into store
    Json(p)
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses((status = 200, description = "Get project by ID", body = Project))
)]
pub async fn get_project(State(state): State<AppState>, Path(_id): Path<String>) -> Json<Project> {
    let p = Project {
        id: ares_core::ProjectId(ares_core::id::new_id()),
        name: "Mock Project".into(),
        description: "Mock Description".into(),
        root_path: state.config.project_path.clone(),
        primary_language: ares_core::Language::Rust.as_str().into(),
        domain: "Mock Domain".into(),
        maturity: ares_core::ProjectMaturity::Greenfield,
        created_at: chrono::Utc::now().timestamp_micros(),
        updated_at: chrono::Utc::now().timestamp_micros(),
        deleted_at: None,
    };
    Json(p)
}
