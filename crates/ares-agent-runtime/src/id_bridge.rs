//! ID Bridge — explicit conversions between `ares-agent-runtime` Uuid-based IDs
//! and `ares-core` String-based IDs.
//!
//! Every cross-crate handoff should go through these helpers so that
//! UUID ↔ String mismatches are caught at compile time rather than at runtime.

use crate::models::{AgentId, ExecutionId, MissionId, TaskId};

// ── To core (Uuid → String) ────────────────────────────────────────

/// Convert a runtime `MissionId` to an `ares_core` string ID.
pub fn mission_id_to_core(id: &MissionId) -> String {
    id.0.to_string()
}

/// Convert a runtime `TaskId` to an `ares_core` string ID.
pub fn task_id_to_core(id: &TaskId) -> String {
    id.0.to_string()
}

/// Convert a runtime `AgentId` to an `ares_core` string ID.
pub fn agent_id_to_core(id: &AgentId) -> String {
    id.0.to_string()
}

/// Convert a runtime `ExecutionId` to an `ares_core` string ID.
pub fn execution_id_to_core(id: &ExecutionId) -> String {
    id.0.to_string()
}

// ── From core (String → Uuid) ──────────────────────────────────────

/// Parse a core string ID into a runtime `MissionId`.
/// Returns `None` if the string is not a valid UUID.
pub fn core_to_mission_id(s: &str) -> Option<MissionId> {
    uuid::Uuid::parse_str(s).ok().map(MissionId)
}

/// Parse a core string ID into a runtime `TaskId`.
pub fn core_to_task_id(s: &str) -> Option<TaskId> {
    uuid::Uuid::parse_str(s).ok().map(TaskId)
}

/// Parse a core string ID into a runtime `AgentId`.
pub fn core_to_agent_id(s: &str) -> Option<AgentId> {
    uuid::Uuid::parse_str(s).ok().map(AgentId)
}

/// Parse a core string ID into a runtime `ExecutionId`.
pub fn core_to_execution_id(s: &str) -> Option<ExecutionId> {
    uuid::Uuid::parse_str(s).ok().map(ExecutionId)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mission_id_roundtrip() {
        let original = MissionId::new();
        let core_str = mission_id_to_core(&original);
        let back = core_to_mission_id(&core_str).expect("roundtrip failed");
        assert_eq!(original, back);
    }

    #[test]
    fn task_id_roundtrip() {
        let original = TaskId::new();
        let core_str = task_id_to_core(&original);
        let back = core_to_task_id(&core_str).expect("roundtrip failed");
        assert_eq!(original, back);
    }

    #[test]
    fn agent_id_roundtrip() {
        let original = AgentId::new();
        let core_str = agent_id_to_core(&original);
        let back = core_to_agent_id(&core_str).expect("roundtrip failed");
        assert_eq!(original, back);
    }

    #[test]
    fn execution_id_roundtrip() {
        let original = ExecutionId::new();
        let core_str = execution_id_to_core(&original);
        let back = core_to_execution_id(&core_str).expect("roundtrip failed");
        assert_eq!(original, back);
    }

    #[test]
    fn invalid_string_returns_none() {
        assert!(core_to_mission_id("not-a-uuid").is_none());
        assert!(core_to_task_id("").is_none());
        assert!(core_to_agent_id("12345").is_none());
        assert!(core_to_execution_id("xyz").is_none());
    }

    #[test]
    fn core_id_format_is_standard_uuid() {
        let id = MissionId::new();
        let s = mission_id_to_core(&id);
        // Standard UUID format: 8-4-4-4-12 hex chars
        assert_eq!(s.len(), 36);
        assert_eq!(s.chars().filter(|c| *c == '-').count(), 4);
    }
}
