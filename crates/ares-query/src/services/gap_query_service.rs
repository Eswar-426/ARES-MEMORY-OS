use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GapResponse {
    pub architectural_gaps: Vec<String>,
    pub decision_gaps: Vec<String>,
}

pub struct GapQueryService;

impl GapQueryService {
    pub fn execute(project_id: &ProjectId) -> QueryResult<GapResponse> {
        QueryResult {
            data: GapResponse {
                architectural_gaps: vec![],
                decision_gaps: vec![],
            },
            evidence: QueryEvidence { node_ids: vec![] },
            metadata: QueryMetadata {
                confidence: 0.91,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
