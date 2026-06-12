use ares_agent_runtime::models::{AgentId, TaskId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a delegation record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DelegationId(pub Uuid);

impl DelegationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for DelegationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Status of a delegation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelegationStatus {
    Pending,
    Accepted,
    InProgress,
    Completed,
    Failed,
    Reassigned,
    Escalated,
    Cancelled,
}

/// A record of a task delegation from one agent to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRecord {
    pub id: DelegationId,
    pub task_id: TaskId,
    pub from_agent: AgentId,
    pub to_agent: AgentId,
    pub depth: u32,
    pub status: DelegationStatus,
    pub reason: String,
    pub result: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

impl DelegationRecord {
    pub fn new(
        task_id: TaskId,
        from_agent: AgentId,
        to_agent: AgentId,
        depth: u32,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: DelegationId::new(),
            task_id,
            from_agent,
            to_agent,
            depth,
            status: DelegationStatus::Pending,
            reason: reason.into(),
            result: None,
            created_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        }
    }

    pub fn accept(&mut self) {
        self.status = DelegationStatus::Accepted;
    }

    pub fn start(&mut self) {
        self.status = DelegationStatus::InProgress;
    }

    pub fn complete(&mut self, result: impl Into<String>) {
        self.status = DelegationStatus::Completed;
        self.result = Some(result.into());
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }

    pub fn fail(&mut self, reason: impl Into<String>) {
        self.status = DelegationStatus::Failed;
        self.result = Some(reason.into());
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }

    pub fn escalate(&mut self) {
        self.status = DelegationStatus::Escalated;
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            DelegationStatus::Completed | DelegationStatus::Failed | DelegationStatus::Cancelled
        )
    }
}
