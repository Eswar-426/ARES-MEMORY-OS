use crate::models::{AgentId, MissionId, TaskId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
    MissionCreated(MissionId),
    MissionStarted(MissionId),
    MissionPaused(MissionId),
    MissionResumed(MissionId),
    MissionCompleted(MissionId),
    MissionFailed(MissionId, String),
    MissionCancelled(MissionId),

    TaskAssigned(TaskId, AgentId),
    TaskStarted(TaskId, AgentId),
    TaskCompleted(TaskId, AgentId),
    TaskFailed(TaskId, AgentId, String),

    AgentAllocated(AgentId),
    AgentReleased(AgentId),
    AgentRecovered(AgentId),

    ResourceExhausted(String),
}

pub struct EventBus {
    // Can be backed by tokio::sync::broadcast or integrated with ares-events
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn publish(&self, _event: RuntimeEvent) {
        // Emit event to subscribers
    }
}
