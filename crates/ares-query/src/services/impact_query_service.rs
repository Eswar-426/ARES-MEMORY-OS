use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImpactResponse {
    pub breaking_nodes: Vec<String>,
    pub risk_level: String,
}

pub struct ImpactQueryService;

impl ImpactQueryService {
    pub fn execute(project_id: &ProjectId, node_id: &str) -> QueryResult<ImpactResponse> {
        QueryResult {
            data: ImpactResponse {
                breaking_nodes: vec![],
                risk_level: "High".into(),
            },
            evidence: QueryEvidence {
                node_ids: vec![node_id.to_string()],
            },
            metadata: QueryMetadata {
                confidence: 0.88,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
