use ares_agent_runtime::models::AgentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique conflict identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConflictId(pub Uuid);

impl ConflictId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for ConflictId {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of conflict between agents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Multiple agents need the same resource.
    ResourceContention,
    /// Agents produce conflicting plans.
    ContradictoryPlans,
    /// Agents working on the same task unknowingly.
    DuplicateWork,
    /// Competing priorities between teams.
    PriorityClash,
}

/// Resolution strategy applied to a conflict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Resolution {
    /// Higher priority agent wins.
    PriorityBased(AgentId),
    /// Merge both solutions.
    Merge(String),
    /// Escalate to parent/leader.
    Escalate,
    /// One agent yields.
    Yield(AgentId),
    /// Conflict was not resolvable.
    Unresolved(String),
}

/// State of a conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictState {
    Detected,
    UnderReview,
    Resolved,
    Unresolvable,
}

/// A conflict between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: ConflictId,
    pub conflict_type: ConflictType,
    pub involved_agents: Vec<AgentId>,
    pub description: String,
    pub state: ConflictState,
    pub resolution: Option<Resolution>,
    pub detected_at: i64,
    pub resolved_at: Option<i64>,
}

impl Conflict {
    pub fn new(
        conflict_type: ConflictType,
        involved_agents: Vec<AgentId>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: ConflictId::new(),
            conflict_type,
            involved_agents,
            description: description.into(),
            state: ConflictState::Detected,
            resolution: None,
            detected_at: chrono::Utc::now().timestamp(),
            resolved_at: None,
        }
    }

    pub fn resolve(&mut self, resolution: Resolution) {
        self.resolution = Some(resolution);
        self.state = ConflictState::Resolved;
        self.resolved_at = Some(chrono::Utc::now().timestamp());
    }

    pub fn mark_unresolvable(&mut self, reason: impl Into<String>) {
        self.resolution = Some(Resolution::Unresolved(reason.into()));
        self.state = ConflictState::Unresolvable;
        self.resolved_at = Some(chrono::Utc::now().timestamp());
    }

    pub fn is_resolved(&self) -> bool {
        matches!(
            self.state,
            ConflictState::Resolved | ConflictState::Unresolvable
        )
    }
}
