use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateSource {
    pub id: String,
    pub candidate_id: String,
    pub source_type: String, // e.g., "commit", "pr", "file"
    pub source_id: String, // e.g., commit hash, file path
    pub confidence: f64,
}
