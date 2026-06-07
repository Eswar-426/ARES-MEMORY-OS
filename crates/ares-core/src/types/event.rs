use crate::id::{EventId, ProjectId};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────
// Event types
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Memory lifecycle
    MemoryCreated,
    MemoryUpdated,
    MemoryDeleted,
    MemoryVersionCreated,

    // Decision lifecycle
    DecisionCreated,
    DecisionUpdated,
    DecisionSuperseded,
    DecisionReviewDue,

    // Scanner
    ScannerRunStarted,
    ScannerFileParsed,
    ScannerRunCompleted,
    ScannerRunFailed,
    ScannerChangeDetected,

    // Graph
    GraphNodeCreated,
    GraphEdgeCreated,
    GraphContradictionDetected,

    // Agent
    AgentSessionStarted,
    AgentActionLogged,
    AgentSessionEnded,

    // System
    ProjectInitialized,
    ProjectUpdated,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MemoryCreated => "memory.created",
            Self::MemoryUpdated => "memory.updated",
            Self::MemoryDeleted => "memory.deleted",
            Self::MemoryVersionCreated => "memory.version_created",
            Self::DecisionCreated => "decision.created",
            Self::DecisionUpdated => "decision.updated",
            Self::DecisionSuperseded => "decision.superseded",
            Self::DecisionReviewDue => "decision.review_due",
            Self::ScannerRunStarted => "scanner.run_started",
            Self::ScannerFileParsed => "scanner.file_parsed",
            Self::ScannerRunCompleted => "scanner.run_completed",
            Self::ScannerRunFailed => "scanner.run_failed",
            Self::ScannerChangeDetected => "scanner.change_detected",
            Self::GraphNodeCreated => "graph.node_created",
            Self::GraphEdgeCreated => "graph.edge_created",
            Self::GraphContradictionDetected => "graph.contradiction_detected",
            Self::AgentSessionStarted => "agent.session_started",
            Self::AgentActionLogged => "agent.action_logged",
            Self::AgentSessionEnded => "agent.session_ended",
            Self::ProjectInitialized => "project.initialized",
            Self::ProjectUpdated => "project.updated",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    #[default]
    Agent,
    Scanner,
    User,
    Mcp,
}

impl EventSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Scanner => "scanner",
            Self::User => "user",
            Self::Mcp => "mcp",
        }
    }
}

impl std::str::FromStr for EventSource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "agent" => Ok(Self::Agent),
            "scanner" => Ok(Self::Scanner),
            "user" => Ok(Self::User),
            "mcp" => Ok(Self::Mcp),
            other => Err(format!("Unknown event source: {other}")),
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Event envelope
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AresEvent {
    pub id: EventId,
    pub event_type: EventType,
    pub project_id: Option<ProjectId>,
    pub payload: serde_json::Value,
    pub source: EventSource,
    /// Unix microseconds
    pub created_at: i64,
}

impl AresEvent {
    pub fn new(
        event_type: EventType,
        project_id: Option<ProjectId>,
        payload: serde_json::Value,
        source: EventSource,
    ) -> Self {
        Self {
            id: EventId::new(),
            event_type,
            project_id,
            payload,
            source,
            created_at: now_micros(),
        }
    }
}

/// Current time as Unix microseconds.
pub fn now_micros() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_micros() as i64
}
