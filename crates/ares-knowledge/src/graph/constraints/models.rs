use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub constraint_type: String,
    pub rules: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
