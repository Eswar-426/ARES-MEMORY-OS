use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LifecycleDecayResponse {
    pub decay_rate: f64,
    pub active_signals: usize,
}

pub struct LifecycleDecayQueryService;

impl LifecycleDecayQueryService {
    pub fn execute(project_id: &ProjectId, node_id: &str) -> QueryResult<LifecycleDecayResponse> {
        QueryResult {
            data: LifecycleDecayResponse {
                decay_rate: 0.05,
                active_signals: 3,
            },
            evidence: QueryEvidence {
                node_ids: vec![node_id.to_string()],
            },
            metadata: QueryMetadata {
                confidence: 0.88,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
