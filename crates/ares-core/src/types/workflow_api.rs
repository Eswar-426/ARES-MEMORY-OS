use crate::{AgentId, EventId, ExecutionId, WorkflowId, WorkflowStatus};
use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

/// Base response for paginated results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "utoipa", aliases(PageResponseExecutionSummary = PageResponse<ExecutionSummary>))]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

/// Base request for paginated list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct PageRequest {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

// ─────────────────────────────────────────────────────────────────
// Workflows & Versions
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct WorkflowRunRequest {
    pub workflow_id: WorkflowId,
    pub workflow_version_id: String,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct WorkflowSummary {
    pub id: WorkflowId,
    pub name: String,
    pub description: String,
    pub current_version: u32,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct WorkflowVersionSummary {
    pub id: String,
    pub workflow_id: WorkflowId,
    pub version: u32,
    pub created_at: i64,
}

// ─────────────────────────────────────────────────────────────────
// Execution Search & Details
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "utoipa", derive(ToSchema, utoipa::IntoParams))]
pub struct ExecutionSearchRequest {
    pub workflow_id: Option<String>,
    pub status: Option<String>,
    pub agent_id: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct ExecutionSummary {
    pub id: ExecutionId,
    pub workflow_version_id: String,
    pub status: WorkflowStatus,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct ExecutionDetails {
    pub id: ExecutionId,
    pub workflow_version_id: String,
    pub status: WorkflowStatus,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
    pub state_json: Option<String>,
}

/// Optimistic concurrency control for state mutations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct ExecutionVersion {
    pub execution_id: ExecutionId,
    pub version: u64,
}

// ─────────────────────────────────────────────────────────────────
// Replay & Audit
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct ReplayVerification {
    pub expected_checksum: String,
    pub actual_checksum: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct ReplayReport {
    pub execution_id: ExecutionId,
    pub events_replayed: u64,
    pub verification: ReplayVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct ReplayAuditEntry {
    pub replay_id: String,
    pub execution_id: ExecutionId,
    pub requested_by: String,
    pub started_at: i64,
    pub completed_at: i64,
    pub events_replayed: usize,
    pub checksum_verified: bool,
}

// ─────────────────────────────────────────────────────────────────
// Analytics & Agents
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct WorkflowAnalyticsReport {
    pub total_executions: u64,
    pub running_executions: u64,
    pub completed_executions: u64,
    pub failed_executions: u64,
    pub p50_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub p99_duration_ms: f64,
    pub retry_rate: f64,
    pub failure_rate: f64,
    pub compensation_rate: f64,
    pub dead_letter_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct AgentSummary {
    pub id: AgentId,
    pub name: String,
    pub is_available: bool,
    pub health_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct AgentPerformanceReport {
    pub id: AgentId,
    pub total_tasks: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
}

// ─────────────────────────────────────────────────────────────────
// Visualization & Timeline
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct WorkflowGraphResponse {
    pub workflow_version_id: String,
    pub mermaid: Option<String>,
    pub graph_json: serde_json::Value,
    pub visualization_truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct WorkflowTimelineEvent {
    pub id: EventId,
    pub execution_id: ExecutionId,
    pub sequence_number: u64,
    pub schema_version: u32,
    pub source_component: String,
    pub event_type: String,
    pub timestamp: i64,
    pub payload: serde_json::Value,
}

// ─────────────────────────────────────────────────────────────────
// State Transitions
// ─────────────────────────────────────────────────────────────────

pub trait WorkflowStateValidator {
    fn can_pause(status: WorkflowStatus) -> bool;
    fn can_resume(status: WorkflowStatus) -> bool;
    fn can_cancel(status: WorkflowStatus) -> bool;
    fn can_retry(status: WorkflowStatus) -> bool;
}

impl WorkflowStateValidator for WorkflowStatus {
    fn can_pause(status: WorkflowStatus) -> bool {
        matches!(status, WorkflowStatus::Running)
    }

    fn can_resume(status: WorkflowStatus) -> bool {
        matches!(status, WorkflowStatus::Paused)
    }

    fn can_cancel(status: WorkflowStatus) -> bool {
        matches!(
            status,
            WorkflowStatus::Pending | WorkflowStatus::Running | WorkflowStatus::Paused
        )
    }

    fn can_retry(status: WorkflowStatus) -> bool {
        matches!(status, WorkflowStatus::Failed)
    }
}
