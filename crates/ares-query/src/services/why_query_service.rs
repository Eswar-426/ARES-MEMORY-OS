use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WhyResponse {
    pub explanation: String,
}

pub struct WhyQueryService;

impl WhyQueryService {
    pub fn execute(project_id: &ProjectId, node_id: &str) -> QueryResult<WhyResponse> {
        QueryResult {
            data: WhyResponse {
                explanation: format!(
                    "Node {} exists to fulfill foundational architectural constraints.",
                    node_id
                ),
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
