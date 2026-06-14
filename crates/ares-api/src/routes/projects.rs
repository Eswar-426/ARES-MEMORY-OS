use ares_app::AppState;
use ares_core::{Project, ProjectId, ProjectMaturity};
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
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub domain: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    responses((status = 200, description = "List registered projects", body = ProjectListResponse))
)]
pub async fn list_projects(State(state): State<AppState>) -> Json<ProjectListResponse> {
    match state.project_repo.list_all() {
        Ok(projects) => Json(ProjectListResponse { projects }),
        Err(_) => Json(ProjectListResponse { projects: vec![] }),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/projects",
    request_body = CreateProjectRequest,
    responses((status = 200, description = "Project created", body = Project))
)]
pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<Project>, axum::http::StatusCode> {
    let now = chrono::Utc::now().timestamp_micros();
    let project = Project {
        id: ProjectId::new(),
        name: payload.name,
        description: payload.description.unwrap_or_default(),
        root_path: payload.path,
        primary_language: payload.primary_language.unwrap_or_else(|| "unknown".into()),
        domain: payload.domain.unwrap_or_default(),
        maturity: ProjectMaturity::Greenfield,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };

    match state.project_repo.create(&project) {
        Ok(created) => Ok(Json(created)),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses((status = 200, description = "Get project by ID", body = Project))
)]
pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, axum::http::StatusCode> {
    let project_id = ProjectId::from(id);
    match state.project_repo.get_by_id(&project_id) {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err(axum::http::StatusCode::NOT_FOUND),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
