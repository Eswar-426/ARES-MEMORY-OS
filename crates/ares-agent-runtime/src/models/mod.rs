use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct AgentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct MissionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    Architect,
    Researcher,
    Coder,
    Tester,
    Reviewer,
    Security,
    Documentation,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MissionState {
    #[default]
    Pending,
    Planning,
    Executing,
    Verifying,
    Recovering,
    Waiting,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AgentState {
    #[default]
    Created,
    Ready,
    Running,
    Waiting,
    Blocked,
    Recovering,
    Completed,
    Failed,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionContext {
    pub mission_id: Option<MissionId>,
    pub execution_id: Option<ExecutionId>,
    pub parent_task_id: Option<TaskId>,
    pub task_id: Option<TaskId>,
    pub variables: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            mission_id: None,
            execution_id: None,
            parent_task_id: None,
            task_id: None,
            variables: HashMap::new(),
            created_at: Utc::now(),
        }
    }
}

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl MissionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for ExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}
