use ares_app::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::Value;

pub async fn strict_enforcement_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = req.headers().clone();
    let is_strict = headers
        .get("X-ARES-STRICT")
        .map(|v| v.as_bytes() == b"true")
        .unwrap_or(false);

    if !is_strict || req.method() != axum::http::Method::POST {
        return Ok(next.run(req).await);
    }

    let uri_path = req.uri().path().to_string();
    tracing::info!("Strict Enforcement Middleware hit: {}", uri_path);
    
    let (parts, body) = req.into_parts();
    
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(collected) => collected,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
    let req_for_next = Request::from_parts(parts, Body::from(bytes.clone()));

    // We only intercept specific paths for strict enforcement
    if uri_path.ends_with("/memory/create") || uri_path.ends_with("/knowledge/entities") {
        if let Ok(payload) = serde_json::from_slice::<Value>(&bytes) {
            let entity_type = payload.get("entity_type").and_then(|v| v.as_str())
                .or_else(|| payload.get("memory_type").and_then(|v| v.as_str()));
                
            if let Some(entity_type_str) = entity_type {
                if entity_type_str.eq_ignore_ascii_case("requirement") 
                    || entity_type_str.eq_ignore_ascii_case("decision")
                    || entity_type_str.eq_ignore_ascii_case("feature") {
                    
                    let proj_path = std::path::PathBuf::from(&state.config.project_path);
                    let governance = ares_governance::GovernanceFacade::new(state.store.clone(), proj_path);
                    
                    let active_policies = governance.get_active_policies().await;
                    let exemptions = governance.get_exemptions().await.unwrap_or_default();
                    
                    let kg_store = ares_knowledge_graph::store::KnowledgeGraphStore::new(std::sync::Arc::new(state.store.clone()));
                    let base_graph = kg_store.export_graph().unwrap_or(ares_knowledge_graph::models::KnowledgeGraph { nodes: vec![], edges: vec![] });
                    
                    let mut base_nodes = Vec::new();
                    let base_edges = Vec::new(); 
                    
                    // Convert from KnowledgeNode to GraphNode
                    for n in &base_graph.nodes {
                        base_nodes.push(ares_core::types::node::GraphNode {
                            id: ares_core::NodeId::from(n.id.as_str()),
                            project_id: ares_core::ProjectId::from("DEFAULT"),
                            node_type: n.node_type.to_string().parse().unwrap_or(ares_core::types::node::NodeType::Concept),
                            label: n.name.clone(),
                            properties: serde_json::Value::Null,
                            file_path: None,
                            created_at: 0,
                            updated_at: 0,
                            deleted_at: None,
                        });
                    }
                    
                    let fake_id = uuid::Uuid::new_v4().to_string();
                    tracing::info!("Parsing entity_type_str: '{}', lowercase: '{}'", entity_type_str, entity_type_str.to_lowercase());
                    let node_type: ares_core::types::node::NodeType = entity_type_str.to_lowercase().parse().unwrap_or(ares_core::types::node::NodeType::Concept);
                    let label = payload.get("name").or_else(|| payload.get("title")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let project_id_str = payload.get("project_id").and_then(|v| v.as_str()).unwrap_or("DEFAULT");
                    
                    let mutation = ares_governance::mutation_simulator::GraphMutation::AddNode(ares_core::types::node::GraphNode {
                        id: ares_core::NodeId::from(fake_id.as_str()),
                        project_id: ares_core::ProjectId::from(project_id_str),
                        node_type,
                        label,
                        properties: serde_json::Value::Null,
                        file_path: None,
                        created_at: 0,
                        updated_at: 0,
                        deleted_at: None,
                    });
                    
                    let provider = ares_governance::mutation_simulator::VirtualGraphProvider::new(base_nodes, base_edges, mutation);
                    
                    let evaluation = ares_governance::strict_evaluation::StrictEvaluationEngine::evaluate(
                        &ares_core::ProjectId::from(project_id_str),
                        &ares_core::NodeId::from(fake_id.as_str()),
                        provider,
                        &active_policies,
                        &exemptions
                    );
                    
                    match evaluation {
                        Ok(res) => {
                            tracing::info!("Strict Mode Evaluation for node type {}: allowed={}, violations={}", entity_type_str, res.allowed, res.violations.len());
                            if !res.allowed {
                                let explanations: Vec<_> = res.violations.iter()
                                    .map(|v| ares_governance::explainability::explainer::GovernanceExplainer::summarize(v))
                                    .collect();
                                let violations_json = serde_json::to_string(&explanations).unwrap_or_default();
                                let risk_str = format!("{:?}", res.risk_level);
                                return Ok(Response::builder()
                                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                                    .header("Content-Type", "application/json")
                                    .body(Body::from(format!(r#"{{"status":"blocked","risk_level":"{}","violations":{}}}"#, risk_str, violations_json)))
                                    .unwrap()
                                    .into_response());
                            } else if res.outcome == ares_governance::models::GovernanceOutcome::Warn {
                                let mut response = next.run(req_for_next).await;
                                let violations_json = serde_json::to_string(&res.violations).unwrap_or_default();
                                response.headers_mut().insert("X-ARES-WARNINGS", axum::http::HeaderValue::from_str(&violations_json).unwrap());
                                return Ok(response);
                            }
                        },
                        Err(e) => {
                            return Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from(format!(r#"{{"status":"error","message":"Strict Mode evaluation failed: {}"}}"#, e)))
                                .unwrap()
                                .into_response());
                        }
                    }
                }
            }
        }
    }

    Ok(next.run(req_for_next).await)
}
