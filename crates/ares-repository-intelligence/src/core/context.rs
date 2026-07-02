use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Layer 1: RepositoryContext — Immutable input to every planner run
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub name: String,
    pub root_path: String,
}

/// Immutable snapshot of the repository state at execution time.
/// Ties every planner run to a specific point in the repository's history,
/// enabling deterministic replay months later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySnapshot {
    pub repository_id: String,
    pub branch: String,
    pub commit_hash: String,
    pub ingest_version: u32,
    pub graph_version: u32,
}

impl Default for RepositorySnapshot {
    fn default() -> Self {
        Self {
            repository_id: String::new(),
            branch: "main".to_string(),
            commit_hash: String::new(),
            ingest_version: 0,
            graph_version: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceContext {
    pub workspace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryPoint {
    Chat,
    Doctor,
    Graph,
    SelfTest,
    CLI,
    API,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CachePolicy {
    UseCache,
    BypassCache,
    UpdateCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    Direct,
    Intelligence,
    Conversation,
    Batch,
    Background,
}

/// Per-execution policy that controls planner behavior without special-case code.
/// Different frontends can customize behavior:
///   - Dashboard → shallow graph, cached
///   - Graph Explorer → deep traversal
///   - Chat → maximum context
///   - CLI → minimal latency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub parallelism: u32,
    pub timeout_ms: u64,
    pub max_depth: u32,
    pub cache_policy: CachePolicy,
    pub replay_enabled: bool,
    pub trace_enabled: bool,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            parallelism: 4,
            timeout_ms: 30000,
            max_depth: 5,
            cache_policy: CachePolicy::UseCache,
            replay_enabled: true,
            trace_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub started_at: u64,
    pub requested_by: String,
    pub entry_point: EntryPoint,
    pub execution_mode: ExecutionMode,
    pub streaming: bool,
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub query: String,
    pub parameters: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryContext {
    pub repository: RepositoryInfo,
    pub snapshot: RepositorySnapshot,
    pub workspace: WorkspaceContext,
    pub execution: ExecutionContext,
    pub policy: ExecutionPolicy,
    pub request: RequestContext,
}
