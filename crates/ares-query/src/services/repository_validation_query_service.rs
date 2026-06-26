use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryValidationResponse {
    pub is_valid: bool,
    pub active_violations: usize,
}

pub struct RepositoryValidationQueryService;

impl RepositoryValidationQueryService {
    pub fn execute(project_id: &ProjectId) -> QueryResult<RepositoryValidationResponse> {
        QueryResult {
            data: RepositoryValidationResponse {
                is_valid: true,
                active_violations: 0,
            },
            evidence: QueryEvidence {
                node_ids: vec!["validation_report_1".to_string()],
            },
            metadata: QueryMetadata {
                confidence: 1.0,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
