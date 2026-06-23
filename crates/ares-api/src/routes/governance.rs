use ares_app::AppState;
use ares_core::{NodeId, ProjectId};
use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

#[utoipa::path(
    get,
    path = "/api/v1/governance/compliance/{project_id}/{node_id}",
    responses(
        (status = 200, description = "Compliance results", body = Vec<ares_governance::models::ComplianceResult>),
        (status = 404, description = "Node not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn compliance(
    State(state): State<AppState>,
    Path((project_id, node_id)): Path<(String, String)>,
) -> Result<Json<Vec<ares_governance::models::ComplianceResult>>, axum::http::StatusCode> {
    info!(%project_id, %node_id, "Compliance evaluation requested");

    // In a real app we'd get MemoryFacade from state, but currently AppState might not have GovernanceFacade inside it.
    // Wait, the user's codebase creates MemoryFacade inline in create_router and doesn't store it in AppState.
    // I'll need to instantiate GovernanceFacade here OR add it to AppState.
    // It's better to add MemoryFacade or GovernanceFacade to AppState or create it on the fly.
    // Let's create it on the fly for now, as AppState just has `store` and `config`.

    let governance = ares_governance::GovernanceFacade::new(
        state.store.clone(),
        std::path::PathBuf::from(state.config.project_path.clone()),
    );

    match governance
        .is_compliant(&ProjectId::from(project_id), &NodeId::from(node_id))
        .await
    {
        Ok(results) => Ok(Json(results)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/governance/certification/{project_id}",
    responses(
        (status = 200, description = "Governance certification", body = ares_governance::models::GovernanceCertification),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn certification(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ares_governance::models::GovernanceCertification>, axum::http::StatusCode> {
    let governance = ares_governance::GovernanceFacade::new(
        state.store.clone(),
        std::path::PathBuf::from(state.config.project_path.clone()),
    );

    match governance
        .get_certification(&ProjectId::from(project_id))
        .await
    {
        Ok(cert) => Ok(Json(cert)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/governance/scorecard/{project_id}",
    responses(
        (status = 200, description = "Governance scorecard", body = ares_governance::models::GovernanceScorecard),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn scorecard(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ares_governance::models::GovernanceScorecard>, axum::http::StatusCode> {
    let governance = ares_governance::GovernanceFacade::new(
        state.store.clone(),
        std::path::PathBuf::from(state.config.project_path.clone()),
    );

    match governance.get_scorecard(&ProjectId::from(project_id)).await {
        Ok(scorecard) => Ok(Json(scorecard)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/governance/policies/{project_id}",
    responses(
        (status = 200, description = "Active policies", body = Vec<ares_governance::models::PolicyVersion>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn policies(
    State(state): State<AppState>,
    Path(_project_id): Path<String>,
) -> Result<Json<Vec<ares_governance::models::PolicyVersion>>, axum::http::StatusCode> {
    let governance = ares_governance::GovernanceFacade::new(
        state.store.clone(),
        std::path::PathBuf::from(state.config.project_path.clone()),
    );

    Ok(Json(governance.get_policies().await))
}

#[utoipa::path(
    get,
    path = "/api/v1/governance/dashboard/{project_id}",
    responses(
        (status = 200, description = "Governance dashboard", body = ares_governance::models::GovernanceDashboard),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn dashboard(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ares_governance::models::GovernanceDashboard>, axum::http::StatusCode> {
    let governance = ares_governance::GovernanceFacade::new(
        state.store.clone(),
        std::path::PathBuf::from(state.config.project_path.clone()),
    );

    match governance.get_dashboard(&ProjectId::from(project_id)).await {
        Ok(dash) => Ok(Json(dash)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/governance/drift/{project_id}",
    responses(
        (status = 200, description = "Policy drift status", body = ares_governance::models::PolicyDriftStatus),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn drift(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ares_governance::models::PolicyDriftStatus>, axum::http::StatusCode> {
    let governance = ares_governance::GovernanceFacade::new(
        state.store.clone(),
        std::path::PathBuf::from(state.config.project_path.clone()),
    );

    match governance.detect_drift(&ProjectId::from(project_id)).await {
        Ok(status) => Ok(Json(status)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/governance/exemptions/{project_id}",
    responses(
        (status = 200, description = "Active policy exemptions", body = Vec<ares_governance::models::PolicyExemption>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn exemptions(
    State(state): State<AppState>,
    Path(_project_id): Path<String>,
) -> Result<Json<Vec<ares_governance::models::PolicyExemption>>, axum::http::StatusCode> {
    let governance = ares_governance::GovernanceFacade::new(
        state.store.clone(),
        std::path::PathBuf::from(state.config.project_path.clone()),
    );

    match governance.get_exemptions().await {
        Ok(exemptions) => Ok(Json(exemptions)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/governance/explain/{violation_id}",
    responses(
        (status = 200, description = "Explanation", body = ares_governance::models::GovernanceExplanation),
        (status = 404, description = "Explanation not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn explain(
    Path(violation_id): Path<String>,
) -> Result<Json<ares_governance::models::GovernanceExplanation>, axum::http::StatusCode> {
    // Generate a dummy compliance violation to feed the explainer,
    // since the explainer operates on rule definitions.
    let fake_violation = ares_governance::models::ComplianceViolation {
        id: violation_id.clone(),
        severity: ares_governance::models::ViolationSeverity::Error,
        policy_name: format!("Policy for {}", violation_id),
        node_id: "unknown".to_string(),
        reason: "Violation explanation requested".to_string(),
        supporting_nodes: vec![],
        enforcement: ares_governance::models::EnforcementAction::Block,
        category: ares_governance::models::PolicyCategory::Architecture,
    };

    // In the future this might pull the actual stored violation if it has dynamic evidence.
    let explanation =
        ares_governance::explainability::explainer::GovernanceExplainer::explain(&fake_violation);

    Ok(Json(explanation))
}
