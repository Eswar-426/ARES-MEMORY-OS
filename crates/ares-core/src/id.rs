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
                Self(s.to_lowercase())
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_lowercase())
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

// Week 27 — Requirement Intelligence IDs
define_id!(RequirementId, "requirement");
define_id!(ArchComponentId, "arch_component");
define_id!(CodeArtifactId, "code_artifact");
define_id!(RuntimeMetricId, "runtime_metric");
define_id!(RequirementLinkId, "requirement_link");
define_id!(RequirementRevisionId, "requirement_revision");
define_id!(EvidenceId, "evidence");

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

// Week 18 — World Model & Predictive Planning IDs
define_id!(WorldStateId, "world_state");
define_id!(ScenarioId, "scenario");
define_id!(SimulationId, "simulation");
define_id!(RiskReportId, "risk_report");
define_id!(PredictionId, "prediction");
define_id!(ForecastId, "forecast");

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

/// Normalizes a path string into a canonical node ID for graph storage and lookup.
/// Rules:
/// 1. Strips leading `./` or `.\`
/// 2. Converts `\` to `/`
/// 3. Collapses duplicate separators
/// 4. Never stores absolute paths (if the string happens to be absolute, it removes leading slashes/drives if feasible, but primarily focuses on relative artifact paths)
pub fn canonicalize_node_id(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");

    // Resiliency: if it's an absolute path that starts with the current working directory, strip it
    if let Ok(cwd) = std::env::current_dir() {
        let cwd_str = cwd.to_string_lossy().replace('\\', "/");
        // Case-insensitive check for Windows
        if normalized
            .to_lowercase()
            .starts_with(&cwd_str.to_lowercase())
        {
            let prefix_len = cwd_str.len();
            normalized = normalized[prefix_len..].to_string();
        }
    }

    // Collapse duplicate slashes
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }

    if normalized.starts_with("./") {
        normalized = normalized[2..].to_string();
    } else if normalized.starts_with('/') {
        // Strip leading slash to make it repository-relative if it accidentally got one
        normalized = normalized.trim_start_matches('/').to_string();
    }

    // For Windows absolute paths like C:/foo, ideally we don't have them, but for safety:
    if normalized.contains(":/") {
        let parts: Vec<&str> = normalized.splitn(2, ":/").collect();
        if parts.len() == 2 {
            normalized = parts[1].to_string();
        }
    }

    normalized
}

/// Converts an absolute or relative path into a **workspace-relative** canonical path.
///
/// This is the single source of truth for path normalization across all ARES crates.
/// Every crate (scanner, git-memory, ingestion, knowledge-graph, MCP) should call
/// this function instead of doing ad-hoc path normalization.
///
/// Rules:
/// 1. If `path` starts with `workspace_root`, strip the prefix.
/// 2. Normalize separators (`\` → `/`).
/// 3. Collapse duplicate separators.
/// 4. Strip leading `./`.
///
/// # Examples
/// ```
/// use ares_core::canonical_repo_path;
/// let result = canonical_repo_path("E:\\My Projects\\repo", "E:\\My Projects\\repo\\src\\main.rs");
/// assert_eq!(result, "src/main.rs");
///
/// // Already relative — passes through unchanged after normalization
/// let result = canonical_repo_path("E:\\My Projects\\repo", "src/main.rs");
/// assert_eq!(result, "src/main.rs");
/// ```
pub fn canonical_repo_path(workspace_root: &str, path: &str) -> String {
    // Normalize both to forward slashes for comparison
    let norm_root = workspace_root.replace('\\', "/");
    let mut norm_path = path.replace('\\', "/");

    // Collapse duplicate slashes
    while norm_path.contains("//") {
        norm_path = norm_path.replace("//", "/");
    }

    // Strip the workspace root prefix (with or without trailing slash)
    let root_prefix = if norm_root.ends_with('/') {
        norm_root.clone()
    } else {
        format!("{}/", norm_root)
    };

    if norm_path.starts_with(&root_prefix) {
        norm_path = norm_path[root_prefix.len()..].to_string();
    } else if norm_path == norm_root || norm_path == norm_root.trim_end_matches('/') {
        // Path IS the workspace root
        return String::new();
    }

    // Strip leading ./
    if norm_path.starts_with("./") {
        norm_path = norm_path[2..].to_string();
    }

    // Strip leading /
    norm_path = norm_path.trim_start_matches('/').to_string();

    norm_path
}

#[cfg(test)]
mod canonicalization_tests {
    use super::*;

    #[test]
    fn test_canonicalize_node_id() {
        assert_eq!(canonicalize_node_id(".\\src\\main.rs"), "src/main.rs");
        assert_eq!(canonicalize_node_id("./src/main.rs"), "src/main.rs");
        assert_eq!(
            canonicalize_node_id("src\\memory\\graph.rs"),
            "src/memory/graph.rs"
        );
        assert_eq!(canonicalize_node_id("src/main.rs"), "src/main.rs");
        assert_eq!(canonicalize_node_id(".\\\\src\\\\main.rs"), "src/main.rs");
        assert_eq!(
            canonicalize_node_id("C:\\repo\\src\\main.rs"),
            "repo/src/main.rs"
        );
    }

    #[test]
    fn test_canonical_repo_path_strips_workspace() {
        assert_eq!(
            canonical_repo_path(
                "E:\\My Projects\\ARES_Memory_os",
                "E:\\My Projects\\ARES_Memory_os\\Cargo.toml"
            ),
            "Cargo.toml"
        );
        assert_eq!(
            canonical_repo_path(
                "E:\\My Projects\\ARES_Memory_os",
                "E:\\My Projects\\ARES_Memory_os\\src\\main.rs"
            ),
            "src/main.rs"
        );
    }

    #[test]
    fn test_canonical_repo_path_already_relative() {
        assert_eq!(
            canonical_repo_path("E:\\My Projects\\ARES_Memory_os", "src/main.rs"),
            "src/main.rs"
        );
        assert_eq!(
            canonical_repo_path("E:\\My Projects\\ARES_Memory_os", "Cargo.toml"),
            "Cargo.toml"
        );
    }

    #[test]
    fn test_canonical_repo_path_forward_slash_root() {
        assert_eq!(
            canonical_repo_path("/home/user/repo", "/home/user/repo/src/lib.rs"),
            "src/lib.rs"
        );
    }

    #[test]
    fn test_canonical_repo_path_with_dotslash() {
        assert_eq!(
            canonical_repo_path("E:\\repo", ".\\src\\main.rs"),
            "src/main.rs"
        );
    }
}
