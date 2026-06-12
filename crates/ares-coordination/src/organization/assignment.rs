use ares_agent_runtime::models::{AgentId, AgentRole, TaskId};
use serde::{Deserialize, Serialize};

/// Maps an agent to a specific task with role and priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAssignment {
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub role: AgentRole,
    pub priority: AssignmentPriority,
    pub status: AssignmentStatus,
    pub constraints: Vec<String>,
    pub assigned_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AssignmentPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Reassigned,
    Cancelled,
}

impl AgentAssignment {
    pub fn new(agent_id: AgentId, task_id: TaskId, role: AgentRole) -> Self {
        Self {
            agent_id,
            task_id,
            role,
            priority: AssignmentPriority::Normal,
            status: AssignmentStatus::Pending,
            constraints: Vec::new(),
            assigned_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_priority(mut self, priority: AssignmentPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_constraints(mut self, constraints: Vec<String>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn activate(&mut self) {
        self.status = AssignmentStatus::Active;
    }

    pub fn complete(&mut self) {
        self.status = AssignmentStatus::Completed;
    }

    pub fn fail(&mut self) {
        self.status = AssignmentStatus::Failed;
    }

    pub fn reassign(&mut self, new_agent: AgentId) {
        self.status = AssignmentStatus::Reassigned;
        self.agent_id = new_agent;
        self.status = AssignmentStatus::Pending;
    }

    pub fn cancel(&mut self) {
        self.status = AssignmentStatus::Cancelled;
    }

    pub fn is_active(&self) -> bool {
        self.status == AssignmentStatus::Active
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            AssignmentStatus::Completed | AssignmentStatus::Failed | AssignmentStatus::Cancelled
        )
    }
}
