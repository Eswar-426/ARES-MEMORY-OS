use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LifecycleTrustResponse {
    pub is_trusted: bool,
    pub trust_score: f64,
}

pub struct LifecycleTrustQueryService;

impl LifecycleTrustQueryService {
    pub fn execute(project_id: &ProjectId, node_id: &str) -> QueryResult<LifecycleTrustResponse> {
        QueryResult {
            data: LifecycleTrustResponse {
                is_trusted: true,
                trust_score: 0.98,
            },
            evidence: QueryEvidence {
                node_ids: vec![node_id.to_string()],
            },
            metadata: QueryMetadata {
                confidence: 0.95,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
