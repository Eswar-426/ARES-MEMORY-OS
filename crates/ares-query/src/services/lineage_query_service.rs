use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LineageResponse {
    pub upstream: Vec<String>,
    pub downstream: Vec<String>,
}

pub struct LineageQueryService;

impl LineageQueryService {
    pub fn execute(project_id: &ProjectId, node_id: &str) -> QueryResult<LineageResponse> {
        QueryResult {
            data: LineageResponse {
                upstream: vec![],
                downstream: vec![],
            },
            evidence: QueryEvidence {
                node_ids: vec![node_id.to_string()],
            },
            metadata: QueryMetadata {
                confidence: 0.92,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
