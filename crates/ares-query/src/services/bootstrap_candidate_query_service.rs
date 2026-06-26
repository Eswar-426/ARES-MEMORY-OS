use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BootstrapCandidateResponse {
    pub candidates_inferred: usize,
    pub primary_sources: Vec<String>,
}

pub struct BootstrapCandidateQueryService;

impl BootstrapCandidateQueryService {
    pub fn execute(project_id: &ProjectId) -> QueryResult<BootstrapCandidateResponse> {
        QueryResult {
            data: BootstrapCandidateResponse {
                candidates_inferred: 15,
                primary_sources: vec!["cargo.toml".to_string(), "package.json".to_string()],
            },
            evidence: QueryEvidence {
                node_ids: vec!["candidate_gen_1".to_string()],
            },
            metadata: QueryMetadata {
                confidence: 0.90,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
