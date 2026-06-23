use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEvidence {
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub confidence: f32,
    pub generated_at: DateTime<Utc>,
    pub repository_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub data: T,
    pub evidence: QueryEvidence,
    pub metadata: QueryMetadata,
}
