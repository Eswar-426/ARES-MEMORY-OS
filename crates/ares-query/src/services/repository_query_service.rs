use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryResponse {
    pub purpose: String,
    pub active_capabilities: usize,
}

pub struct RepositoryQueryService;

impl RepositoryQueryService {
    pub fn execute(project_id: &ProjectId) -> QueryResult<RepositoryResponse> {
        QueryResult {
            data: RepositoryResponse {
                purpose: "Repository Memory OS".into(),
                active_capabilities: 5,
            },
            evidence: QueryEvidence { node_ids: vec![] },
            metadata: QueryMetadata {
                confidence: 0.96,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
