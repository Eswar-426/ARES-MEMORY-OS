//! Week 8 — Workflow orchestration types.
//!
//! All types are deterministic, serializable, and designed for SQLite persistence.
//! No HashMap/HashSet — BTreeMap/BTreeSet only for deterministic iteration.

use crate::id::{AgentId, EventId, ExecutionId, StepId, TaskId, WorkflowId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// ─────────────────────────────────────────────────────────────────
// Workflow status (extended state machine)
// ─────────────────────────────────────────────────────────────────

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
    Retrying,
    WaitingDependency,
    TimedOut,
}

impl WorkflowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Paused => "paused",
            Self::Retrying => "retrying",
            Self::WaitingDependency => "waiting_dependency",
            Self::TimedOut => "timed_out",
        }
    }
}

impl std::str::FromStr for WorkflowStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "paused" => Ok(Self::Paused),
            "retrying" => Ok(Self::Retrying),
            "waiting_dependency" => Ok(Self::WaitingDependency),
            "timed_out" => Ok(Self::TimedOut),
            other => Err(format!("Unknown workflow status: {other}")),
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Task priority
// ─────────────────────────────────────────────────────────────────

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    #[default]
    Normal = 2,
    Low = 3,
}

// ─────────────────────────────────────────────────────────────────
// Workflow event types (enumerated, schema-versioned)
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowEventType {
    // Workflow lifecycle
    WorkflowCreated,
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowCancelled,
    WorkflowPaused,
    WorkflowResumed,
    WorkflowTimedOut,

    // Step lifecycle
    StepQueued,
    StepStarted,
    StepCompleted,
    StepFailed,
    StepRetrying,
    StepCompensating,
    StepCompensated,
    StepTimedOut,
    StepSkipped,

    // Agent lifecycle
    AgentAssigned,
    AgentReleased,
    AgentHealthChanged,

    // Snapshot
    SnapshotCreated,

    // Dead letter
    DeadLetterCreated,
}

impl WorkflowEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkflowCreated => "workflow.created",
            Self::WorkflowStarted => "workflow.started",
            Self::WorkflowCompleted => "workflow.completed",
            Self::WorkflowFailed => "workflow.failed",
            Self::WorkflowCancelled => "workflow.cancelled",
            Self::WorkflowPaused => "workflow.paused",
            Self::WorkflowResumed => "workflow.resumed",
            Self::WorkflowTimedOut => "workflow.timed_out",
            Self::StepQueued => "step.queued",
            Self::StepStarted => "step.started",
            Self::StepCompleted => "step.completed",
            Self::StepFailed => "step.failed",
            Self::StepRetrying => "step.retrying",
            Self::StepCompensating => "step.compensating",
            Self::StepCompensated => "step.compensated",
            Self::StepTimedOut => "step.timed_out",
            Self::StepSkipped => "step.skipped",
            Self::AgentAssigned => "agent.assigned",
            Self::AgentReleased => "agent.released",
            Self::AgentHealthChanged => "agent.health_changed",
            Self::SnapshotCreated => "snapshot.created",
            Self::DeadLetterCreated => "dead_letter.created",
        }
    }
}

impl std::str::FromStr for WorkflowEventType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "workflow.created" => Ok(Self::WorkflowCreated),
            "workflow.started" => Ok(Self::WorkflowStarted),
            "workflow.completed" => Ok(Self::WorkflowCompleted),
            "workflow.failed" => Ok(Self::WorkflowFailed),
            "workflow.cancelled" => Ok(Self::WorkflowCancelled),
            "workflow.paused" => Ok(Self::WorkflowPaused),
            "workflow.resumed" => Ok(Self::WorkflowResumed),
            "workflow.timed_out" => Ok(Self::WorkflowTimedOut),
            "step.queued" => Ok(Self::StepQueued),
            "step.started" => Ok(Self::StepStarted),
            "step.completed" => Ok(Self::StepCompleted),
            "step.failed" => Ok(Self::StepFailed),
            "step.retrying" => Ok(Self::StepRetrying),
            "step.compensating" => Ok(Self::StepCompensating),
            "step.compensated" => Ok(Self::StepCompensated),
            "step.timed_out" => Ok(Self::StepTimedOut),
            "step.skipped" => Ok(Self::StepSkipped),
            "agent.assigned" => Ok(Self::AgentAssigned),
            "agent.released" => Ok(Self::AgentReleased),
            "agent.health_changed" => Ok(Self::AgentHealthChanged),
            "snapshot.created" => Ok(Self::SnapshotCreated),
            "dead_letter.created" => Ok(Self::DeadLetterCreated),
            other => Err(format!("Unknown workflow event type: {other}")),
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Workflow event (schema-versioned, sequence-numbered)
// ─────────────────────────────────────────────────────────────────

/// Current schema version for workflow events.
pub const WORKFLOW_EVENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub id: EventId,
    pub execution_id: ExecutionId,
    pub sequence_number: u64,
    pub schema_version: u32,
    pub event_type: WorkflowEventType,
    /// Unix microseconds.
    pub timestamp: i64,
    pub payload: String,
}

// ─────────────────────────────────────────────────────────────────
// Retry policy
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    /// Initial delay in milliseconds (doubles each attempt).
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds.
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 500,
            max_delay_ms: 8_000,
        }
    }
}

impl RetryPolicy {
    /// Compute the back-off delay for the given attempt (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        let delay = self
            .initial_delay_ms
            .saturating_mul(1u64 << attempt.min(16));
        delay.min(self.max_delay_ms)
    }
}

// ─────────────────────────────────────────────────────────────────
// Compensation action (saga pattern)
// ─────────────────────────────────────────────────────────────────

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompensationAction {
    /// No compensation defined.
    #[default]
    None,
    /// Roll back by executing a named workflow step.
    RollbackStep { step_name: String },
    /// Execute a custom handler identified by name.
    CustomHandler {
        handler_name: String,
        params: String,
    },
    /// Simply log and continue (no-op compensation).
    LogAndContinue,
}

// ─────────────────────────────────────────────────────────────────
// Workflow step definition
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepDef {
    pub id: StepId,
    pub name: String,
    pub description: String,
    /// Required capability to execute this step.
    pub required_capability: String,
    /// Dependencies (other step IDs that must complete first).
    pub depends_on: Vec<StepId>,
    /// Step-level timeout in milliseconds.
    pub timeout_ms: Option<u64>,
    pub retry_policy: RetryPolicy,
    pub compensation: CompensationAction,
    pub priority: TaskPriority,
}

// ─────────────────────────────────────────────────────────────────
// Workflow definition (immutable, versioned)
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub workflow_id: WorkflowId,
    pub version: u32,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStepDef>,
    /// Workflow-level timeout in milliseconds.
    pub timeout_ms: Option<u64>,
}

// ─────────────────────────────────────────────────────────────────
// Workflow dependency (for DAG)
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDependency {
    pub from_step: StepId,
    pub to_step: StepId,
}

// ─────────────────────────────────────────────────────────────────
// Execution state (snapshotable, replayable)
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    pub workflow_id: WorkflowId,
    pub execution_id: ExecutionId,
    pub workflow_version_id: String,
    pub current_status: WorkflowStatus,
    pub completed_steps: BTreeSet<TaskId>,
    pub failed_steps: BTreeSet<TaskId>,
    pub pending_steps: BTreeSet<TaskId>,
    pub retry_steps: BTreeSet<TaskId>,
    pub running_steps: BTreeSet<TaskId>,
    pub skipped_steps: BTreeSet<TaskId>,
    pub last_event_sequence: u64,
    /// Step → attempt count.
    pub step_attempts: std::collections::BTreeMap<String, u32>,
    /// Step → assigned agent.
    pub step_agents: std::collections::BTreeMap<String, AgentId>,
    /// Optimistic concurrency control version.
    pub version: u64,
}

impl ExecutionState {
    pub fn new(
        workflow_id: WorkflowId,
        execution_id: ExecutionId,
        workflow_version_id: String,
        initial_steps: BTreeSet<TaskId>,
    ) -> Self {
        Self {
            workflow_id,
            execution_id,
            workflow_version_id,
            current_status: WorkflowStatus::Pending,
            completed_steps: BTreeSet::new(),
            failed_steps: BTreeSet::new(),
            pending_steps: initial_steps,
            retry_steps: BTreeSet::new(),
            running_steps: BTreeSet::new(),
            skipped_steps: BTreeSet::new(),
            last_event_sequence: 0,
            step_attempts: std::collections::BTreeMap::new(),
            step_agents: std::collections::BTreeMap::new(),
            version: 0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Workflow execution snapshot (checksum-verified)
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionSnapshot {
    pub execution_id: ExecutionId,
    pub last_event_sequence: u64,
    /// Unix microseconds.
    pub created_at: i64,
    /// BLAKE3 hash of `state_json` for integrity verification.
    pub checksum: String,
    /// Serialized `ExecutionState`.
    pub state_json: String,
}

// ─────────────────────────────────────────────────────────────────
// Dead letter entry (extended diagnostics)
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    pub execution_id: ExecutionId,
    pub step_id: StepId,
    pub workflow_version_id: String,
    pub step_name: String,
    pub failure_reason: String,
    pub attempt_count: u32,
    pub last_error: String,
    pub last_agent_id: Option<AgentId>,
    pub execution_duration_ms: u64,
    /// Unix microseconds.
    pub failed_at: i64,
    /// Unix microseconds.
    pub created_at: i64,
}

// ─────────────────────────────────────────────────────────────────
// Agent types
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AgentInfo {
    pub id: AgentId,
    pub name: String,
    pub capabilities: Vec<String>,
    pub health: AgentHealth,
    pub performance: AgentPerformance,
    /// Unix microseconds.
    pub registered_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AgentHealth {
    /// 0.0–1.0.
    pub health_score: f64,
    pub is_available: bool,
    /// Unix microseconds of last health check.
    pub last_check: i64,
    pub consecutive_failures: u32,
}

impl Default for AgentHealth {
    fn default() -> Self {
        Self {
            health_score: 1.0,
            is_available: true,
            last_check: 0,
            consecutive_failures: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AgentPerformance {
    pub total_tasks: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    /// Average latency in milliseconds.
    pub avg_latency_ms: f64,
    pub success_rate: f64,
}

impl Default for AgentPerformance {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            avg_latency_ms: 0.0,
            success_rate: 1.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Execution plan
// ─────────────────────────────────────────────────────────────────

/// Output of `create_execution_plan`. Contains the DAG ordering and
/// the workflow definition version that was resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub workflow_id: WorkflowId,
    pub workflow_version_id: String,
    pub version: u32,
    pub execution_order: Vec<StepId>,
    pub definition: WorkflowDefinition,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_policy_exponential_backoff() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.delay_for_attempt(0), 500);
        assert_eq!(policy.delay_for_attempt(1), 1000);
        assert_eq!(policy.delay_for_attempt(2), 2000);
        assert_eq!(policy.delay_for_attempt(3), 4000);
        assert_eq!(policy.delay_for_attempt(4), 8000); // hits max
        assert_eq!(policy.delay_for_attempt(5), 8000); // capped
    }

    #[test]
    fn workflow_status_roundtrip() {
        let statuses = vec![
            WorkflowStatus::Pending,
            WorkflowStatus::Running,
            WorkflowStatus::Completed,
            WorkflowStatus::Failed,
            WorkflowStatus::Cancelled,
            WorkflowStatus::Paused,
            WorkflowStatus::Retrying,
            WorkflowStatus::WaitingDependency,
            WorkflowStatus::TimedOut,
        ];
        for status in statuses {
            let s = status.as_str();
            let parsed: WorkflowStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn event_type_roundtrip() {
        let types = vec![
            WorkflowEventType::WorkflowCreated,
            WorkflowEventType::StepCompleted,
            WorkflowEventType::AgentAssigned,
            WorkflowEventType::SnapshotCreated,
            WorkflowEventType::DeadLetterCreated,
        ];
        for et in types {
            let s = et.as_str();
            let parsed: WorkflowEventType = s.parse().unwrap();
            assert_eq!(parsed, et);
        }
    }

    #[test]
    fn execution_state_deterministic_sets() {
        let state = ExecutionState::new(
            WorkflowId::new(),
            ExecutionId::new(),
            "v1".into(),
            BTreeSet::new(),
        );
        assert_eq!(state.current_status, WorkflowStatus::Pending);
        assert_eq!(state.last_event_sequence, 0);
    }
}
