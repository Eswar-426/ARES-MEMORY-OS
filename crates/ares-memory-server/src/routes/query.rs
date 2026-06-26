use ares_core::ProjectId;
use ares_query::services::{
    ImpactQueryService, LineageQueryService, OwnerQueryService, WhyQueryService,
};
use axum::{extract::rejection::JsonRejection, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Request body for node-based queries.
/// The only server-native DTO; carries no intelligence state.
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeQueryRequest {
    pub project_id: Option<String>,
    pub node_id: String,
}

/// POST /query/why
/// Explains why a node exists in the repository memory graph.
/// Delegates entirely to WhyQueryService in ares-query.
pub async fn query_why(body: Result<Json<NodeQueryRequest>, JsonRejection>) -> Json<Value> {
    match body {
        Ok(Json(req)) => {
            let project_id = ProjectId::from(req.project_id.as_deref().unwrap_or("default"));
            let result = WhyQueryService::execute(&project_id, &req.node_id);
            Json(json!(result))
        }
        Err(e) => Json(json!({ "error": e.body_text() })),
    }
}

/// POST /query/lineage
/// Traces upstream and downstream lineage of a node.
/// Delegates entirely to LineageQueryService in ares-query.
pub async fn query_lineage(body: Result<Json<NodeQueryRequest>, JsonRejection>) -> Json<Value> {
    match body {
        Ok(Json(req)) => {
            let project_id = ProjectId::from(req.project_id.as_deref().unwrap_or("default"));
            let result = LineageQueryService::execute(&project_id, &req.node_id);
            Json(json!(result))
        }
        Err(e) => Json(json!({ "error": e.body_text() })),
    }
}

/// POST /query/impact
/// Analyses the change impact risk for a given node.
/// Delegates entirely to ImpactQueryService in ares-query.
pub async fn query_impact(body: Result<Json<NodeQueryRequest>, JsonRejection>) -> Json<Value> {
    match body {
        Ok(Json(req)) => {
            let project_id = ProjectId::from(req.project_id.as_deref().unwrap_or("default"));
            let result = ImpactQueryService::execute(&project_id, &req.node_id);
            Json(json!(result))
        }
        Err(e) => Json(json!({ "error": e.body_text() })),
    }
}

/// POST /query/owner
/// Returns ownership information for a given node.
/// Delegates entirely to OwnerQueryService in ares-query.
pub async fn query_owner(body: Result<Json<NodeQueryRequest>, JsonRejection>) -> Json<Value> {
    match body {
        Ok(Json(req)) => {
            let project_id = ProjectId::from(req.project_id.as_deref().unwrap_or("default"));
            let result = OwnerQueryService::execute(&project_id, &req.node_id);
            Json(json!(result))
        }
        Err(e) => Json(json!({ "error": e.body_text() })),
    }
}
