use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphVersion {
    pub id: Uuid,
    pub version_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

pub struct VersioningService;

impl VersioningService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_version(&self, name: String, description: Option<String>) -> GraphVersion {
        GraphVersion {
            id: Uuid::now_v7(),
            version_name: name,
            description,
            created_at: Utc::now(),
            created_by: None,
        }
    }
}

impl Default for VersioningService {
    fn default() -> Self {
        Self::new()
    }
}
