use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BootstrapCoverageResponse {
    pub architectural_coverage: f64,
    pub requirement_coverage: f64,
}

pub struct BootstrapCoverageQueryService;

impl BootstrapCoverageQueryService {
    pub fn execute(project_id: &ProjectId) -> QueryResult<BootstrapCoverageResponse> {
        QueryResult {
            data: BootstrapCoverageResponse {
                architectural_coverage: 0.85,
                requirement_coverage: 0.60,
            },
            evidence: QueryEvidence {
                node_ids: vec!["coverage_metrics_1".to_string()],
            },
            metadata: QueryMetadata {
                confidence: 0.95,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
