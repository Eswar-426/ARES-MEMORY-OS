use crate::models::{QueryEvidence, QueryMetadata, QueryResult};
use ares_core::ProjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub completeness_score: f32,
    pub governance_score: f32,
    pub gap_score: f32,
    pub evolution_drift_score: f32,
    pub overall_health: f32,
}

pub struct HealthQueryService;

impl HealthQueryService {
    pub fn execute(project_id: &ProjectId) -> QueryResult<HealthResponse> {
        QueryResult {
            data: HealthResponse {
                completeness_score: 95.0,
                governance_score: 98.0,
                gap_score: 92.0,
                evolution_drift_score: 90.0,
                overall_health: 93.75,
            },
            evidence: QueryEvidence { node_ids: vec![] },
            metadata: QueryMetadata {
                confidence: 0.94,
                generated_at: Utc::now(),
                repository_id: project_id.to_string(),
            },
        }
    }
}
