use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AresEvent {
    WorkflowStarted { workflow_id: String },
    WorkflowCompleted { workflow_id: String },
    WorkflowFailed { workflow_id: String, error: String },
    ExecutionStarted { execution_id: String },
    ExecutionCompleted { execution_id: String },
    ExecutionFailed { execution_id: String, error: String },
    WorkerRegistered { worker_id: String },
    WorkerHeartbeat { worker_id: String },
    WorkerDeregistered { worker_id: String },
    // Add more strictly typed events as needed
}

impl AresEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            AresEvent::WorkflowStarted { .. } => "WorkflowStarted",
            AresEvent::WorkflowCompleted { .. } => "WorkflowCompleted",
            AresEvent::WorkflowFailed { .. } => "WorkflowFailed",
            AresEvent::ExecutionStarted { .. } => "ExecutionStarted",
            AresEvent::ExecutionCompleted { .. } => "ExecutionCompleted",
            AresEvent::ExecutionFailed { .. } => "ExecutionFailed",
            AresEvent::WorkerRegistered { .. } => "WorkerRegistered",
            AresEvent::WorkerHeartbeat { .. } => "WorkerHeartbeat",
            AresEvent::WorkerDeregistered { .. } => "WorkerDeregistered",
        }
    }
}
