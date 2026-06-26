use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LifecycleStatusResponse {
    pub is_stale: bool,
    pub days_since_last_update: u32,
}

pub struct LifecycleStatusQueryService;

impl LifecycleStatusQueryService {
    pub fn execute(project_id: &ProjectId, node_id: &str) -> QueryResult<LifecycleStatusResponse> {
        QueryResult {
            data: LifecycleStatusResponse {
                is_stale: false,
                days_since_last_update: 2,
            },
            evidence: QueryEvidence {
                node_ids: vec![node_id.to_string()],
            },
            metadata: QueryMetadata {
                confidence: 1.0,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
