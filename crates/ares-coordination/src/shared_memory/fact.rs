use ares_agent_runtime::models::AgentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a shared fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId(pub Uuid);

impl FactId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for FactId {
    fn default() -> Self {
        Self::new()
    }
}

/// Category of a shared fact.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactCategory {
    Fact,
    Assumption,
    Plan,
    Observation,
    Result,
    Warning,
}

/// A piece of shared knowledge in the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFact {
    pub id: FactId,
    pub author: AgentId,
    pub category: FactCategory,
    pub key: String,
    pub value: String,
    pub confidence: f64,
    pub retracted: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl SharedFact {
    pub fn new(
        author: AgentId,
        category: FactCategory,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: FactId::new(),
            author,
            category,
            key: key.into(),
            value: value.into(),
            confidence: 1.0,
            retracted: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn retract(&mut self) {
        self.retracted = true;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn update_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn is_active(&self) -> bool {
        !self.retracted
    }
}
