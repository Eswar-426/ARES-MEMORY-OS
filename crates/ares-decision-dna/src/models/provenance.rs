use serde::{Deserialize, Serialize};

use super::UserId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceType {
    Human,
    AI,
    Imported,
    Generated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub source_type: SourceType,
    pub author_id: Option<UserId>,
    pub created_by_agent: Option<String>,
    pub reviewed_by: Option<UserId>,
    pub confidence: f32,
    pub source_system: String, // e.g., "ARES-CLI", "ARES-Planner"
    pub original_commit: Option<String>,
    pub pull_request_url: Option<String>,
    pub evidence_links: Vec<String>,
}
