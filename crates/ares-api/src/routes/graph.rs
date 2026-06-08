use ares_app::AppState;
use ares_core::{
    AnalysisScope, ArchitectureHealthReport, GraphStatistics, ImpactPrediction, KnowledgeCluster,
    NodeId, ProjectId, RiskAssessment, RootCauseAnalysis,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct AnalyzeGraphRequest {
    pub project_id: String,
    pub scope: AnalysisScope,
}

#[derive(Serialize, ToSchema)]
pub struct AnalyzeGraphResponse {
    pub statistics: GraphStatistics,
}

#[derive(Deserialize, ToSchema)]
pub struct PredictImpactRequest {
    pub project_id: String,
    pub target_node: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RootCauseRequest {
    pub project_id: String,
    pub target_node: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ArchitectureReportRequest {
    pub project_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RiskAnalysisRequest {
    pub project_id: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/graph/analyze",
    request_body = AnalyzeGraphRequest,
    responses((status = 200, description = "Analyzed Graph", body = AnalyzeGraphResponse))
)]
pub async fn analyze_graph(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeGraphRequest>,
) -> Result<Json<AnalyzeGraphResponse>, (axum::http::StatusCode, String)> {
    let pid = ProjectId::from(req.project_id);
    let kg = state
        .knowledge_graph_engine
        .build_graph(&pid, req.scope)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(AnalyzeGraphResponse {
        statistics: kg.statistics,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/impact/predict",
    request_body = PredictImpactRequest,
    responses((status = 200, description = "Impact Prediction", body = ImpactPrediction))
)]
pub async fn predict_impact(
    State(state): State<AppState>,
    Json(req): Json<PredictImpactRequest>,
) -> Result<Json<ImpactPrediction>, (axum::http::StatusCode, String)> {
    metrics::counter!("ares_impact_predictions_total").increment(1);
    let pid = ProjectId::from(req.project_id);
    let target = NodeId::from(req.target_node);
    let kg = state
        .knowledge_graph_engine
        .build_graph(&pid, AnalysisScope::Project)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = state
        .impact_prediction_engine
        .predict_change_impact(&kg, &target)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/root-cause",
    request_body = RootCauseRequest,
    responses((status = 200, description = "Root Cause Analysis", body = RootCauseAnalysis))
)]
pub async fn find_root_cause(
    State(state): State<AppState>,
    Json(req): Json<RootCauseRequest>,
) -> Result<Json<RootCauseAnalysis>, (axum::http::StatusCode, String)> {
    metrics::counter!("ares_root_cause_requests_total").increment(1);
    let pid = ProjectId::from(req.project_id);
    let target = NodeId::from(req.target_node);
    let kg = state
        .knowledge_graph_engine
        .build_graph(&pid, AnalysisScope::Project)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = state
        .root_cause_engine
        .find_root_cause(&kg, &target)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/architecture/report",
    request_body = ArchitectureReportRequest,
    responses((status = 200, description = "Architecture Health Report", body = ArchitectureHealthReport))
)]
pub async fn architecture_report(
    State(state): State<AppState>,
    Json(req): Json<ArchitectureReportRequest>,
) -> Result<Json<ArchitectureHealthReport>, (axum::http::StatusCode, String)> {
    metrics::counter!("ares_architecture_reports_total").increment(1);
    let pid = ProjectId::from(req.project_id);
    let kg = state
        .knowledge_graph_engine
        .build_graph(&pid, AnalysisScope::Project)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = state.architectural_analysis_engine.analyze(&kg);
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/risk-analysis",
    request_body = RiskAnalysisRequest,
    responses((status = 200, description = "Risk Assessment", body = RiskAssessment))
)]
pub async fn risk_analysis(
    State(state): State<AppState>,
    Json(req): Json<RiskAnalysisRequest>,
) -> Result<Json<RiskAssessment>, (axum::http::StatusCode, String)> {
    metrics::counter!("ares_risk_analysis_total").increment(1);
    let pid = ProjectId::from(req.project_id);
    let kg = state
        .knowledge_graph_engine
        .build_graph(&pid, AnalysisScope::Project)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = state.risk_engine.assess_risk(&kg);
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/graph/statistics",
    request_body = AnalyzeGraphRequest,
    responses((status = 200, description = "Graph Statistics", body = GraphStatistics))
)]
pub async fn graph_statistics(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeGraphRequest>,
) -> Result<Json<GraphStatistics>, (axum::http::StatusCode, String)> {
    let pid = ProjectId::from(req.project_id);
    let kg = state
        .knowledge_graph_engine
        .build_graph(&pid, req.scope)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(kg.statistics))
}

#[utoipa::path(
    post,
    path = "/api/v1/graph/cluster",
    request_body = AnalyzeGraphRequest,
    responses((status = 200, description = "Graph Clusters", body = Vec<KnowledgeCluster>))
)]
pub async fn graph_clusters(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeGraphRequest>,
) -> Result<Json<Vec<KnowledgeCluster>>, (axum::http::StatusCode, String)> {
    let pid = ProjectId::from(req.project_id);
    let kg = state
        .knowledge_graph_engine
        .build_graph(&pid, req.scope)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let clusters = state.graph_clustering_engine.discover_clusters(&kg);
    Ok(Json(clusters))
}

#[utoipa::path(
    post,
    path = "/api/v1/graph/critical-path",
    request_body = PredictImpactRequest,
    responses((status = 200, description = "Critical Path", body = Vec<NodeId>))
)]
pub async fn graph_critical_path(
    State(state): State<AppState>,
    Json(req): Json<PredictImpactRequest>,
) -> Result<Json<Vec<NodeId>>, (axum::http::StatusCode, String)> {
    let pid = ProjectId::from(req.project_id);
    let target = NodeId::from(req.target_node);
    let kg = state
        .knowledge_graph_engine
        .build_graph(&pid, AnalysisScope::Project)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // We get critical path from root cause analysis and maybe fan in/out
    let result = state
        .root_cause_engine
        .find_root_cause(&kg, &target)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let path: Vec<NodeId> = result
        .evidence_chain
        .into_iter()
        .filter_map(|e| e.node_id)
        .collect();
    Ok(Json(path))
}
