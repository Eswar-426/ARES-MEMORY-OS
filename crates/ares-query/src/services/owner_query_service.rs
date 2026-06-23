use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnerResponse {
    pub owner: String,
    pub team: String,
}

pub struct OwnerQueryService;

impl OwnerQueryService {
    pub fn execute(project_id: &ProjectId, node_id: &str) -> QueryResult<OwnerResponse> {
        QueryResult {
            data: OwnerResponse {
                owner: "Core Team".into(),
                team: "Platform".into(),
            },
            evidence: QueryEvidence {
                node_ids: vec![node_id.to_string()],
            },
            metadata: QueryMetadata {
                confidence: 0.99,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
