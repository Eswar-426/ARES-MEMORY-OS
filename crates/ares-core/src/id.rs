use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────
// Newtype ID wrappers — prevent mixing IDs of different entity types
// ─────────────────────────────────────────────────────────────────
//
// All IDs are UUIDv7 (time-ordered, k-sortable).
// Using newtypes means the compiler catches `memory_id` passed where
// a `project_id` is expected — a very common class of bug.

macro_rules! define_id {
    ($name:ident, $resource:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new() -> Self {
                Self(new_id())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn resource_type() -> &'static str {
                $resource
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::convert::Infallible;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.to_string()))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

/// Generate a new UUIDv7 string.
///
/// UUIDv7 is time-ordered (millisecond precision) and lexicographically
/// sortable, making it ideal for database primary keys without needing
/// separate created_at indexes for ordering.
pub fn new_id() -> String {
    Uuid::now_v7().to_string()
}

define_id!(ProjectId, "project");
define_id!(MemoryId, "memory");
define_id!(DecisionId, "decision");
define_id!(NodeId, "node");
define_id!(EventId, "event");
define_id!(ScanRunId, "scan_run");

// Week 8 — Workflow orchestration IDs
define_id!(WorkflowId, "workflow");
define_id!(ExecutionId, "execution");
define_id!(AgentId, "agent");
define_id!(TaskId, "task");
define_id!(StepId, "step");

// Week 12 — Autonomous Planning IDs
define_id!(GoalId, "goal");
define_id!(PlanId, "plan");
define_id!(PlanCandidateId, "plan_candidate");

// Week 17 — Memory Intelligence IDs
define_id!(EpisodeId, "episode");
define_id!(SemanticMemoryId, "semantic_memory");
define_id!(ClusterId, "cluster");
define_id!(PrincipleId, "principle");
define_id!(ExperienceId, "experience");
define_id!(LessonId, "lesson");

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn new_id_is_valid_uuid() {
        let id = new_id();
        assert!(Uuid::parse_str(&id).is_ok());
    }

    #[test]
    fn new_id_is_uuid_v7() {
        let id = new_id();
        let parsed = Uuid::parse_str(&id).unwrap();
        assert_eq!(parsed.get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn ids_are_time_ordered() {
        let id1 = new_id();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = new_id();
        // UUIDv7 strings are lexicographically time-ordered
        assert!(
            id1 < id2,
            "UUIDv7 ids should be time-ordered: {} < {}",
            id1,
            id2
        );
    }

    #[test]
    fn project_id_and_memory_id_are_distinct_types() {
        let pid = ProjectId::new();
        let mid = MemoryId::new();
        // This would fail to compile if we tried to pass pid where mid is expected:
        // take_memory_id(pid); — compiler error
        assert_ne!(pid.as_str(), mid.as_str()); // UUIDs are unique
    }

    #[test]
    fn id_serializes_as_plain_string() {
        let id = MemoryId::from_str("test-id-123").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""test-id-123""#);
    }
}
