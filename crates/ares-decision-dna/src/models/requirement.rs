use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type RequirementId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequirementStatus {
    Draft,
    Active,
    Fulfilled,
    Obsolete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequirementSource {
    User,
    System,
    Compliance,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: RequirementId,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub status: RequirementStatus,
    pub source: RequirementSource,
    pub created_at: DateTime<Utc>,
}
