use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityResponse {
    pub name: String,
    pub nodes: Vec<String>,
}

pub struct CapabilityQueryService;

impl CapabilityQueryService {
    pub fn execute(project_id: &ProjectId, name: &str) -> QueryResult<CapabilityResponse> {
        QueryResult {
            data: CapabilityResponse {
                name: name.to_string(),
                nodes: vec![],
            },
            evidence: QueryEvidence { node_ids: vec![] },
            metadata: QueryMetadata {
                confidence: 0.90,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
