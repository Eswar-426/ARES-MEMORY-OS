use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LifecycleRevalidationResponse {
    pub requires_revalidation: bool,
    pub reason: Option<String>,
}

pub struct LifecycleRevalidationQueryService;

impl LifecycleRevalidationQueryService {
    pub fn execute(
        project_id: &ProjectId,
        node_id: &str,
    ) -> QueryResult<LifecycleRevalidationResponse> {
        QueryResult {
            data: LifecycleRevalidationResponse {
                requires_revalidation: false,
                reason: None,
            },
            evidence: QueryEvidence {
                node_ids: vec![node_id.to_string()],
            },
            metadata: QueryMetadata {
                confidence: 0.90,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
