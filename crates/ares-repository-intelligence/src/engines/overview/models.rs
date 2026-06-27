use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryOverview {
    pub name: String,
    pub root_path: String,
    pub language: String,
    pub branch: String,
    pub commit: String,
    pub files: usize,
    pub functions: usize,
    pub directories: usize,
    pub modules: usize,
    pub indexed: bool,
    pub last_ingest: String,
    pub is_dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphOverview {
    pub nodes: usize,
    pub edges: usize,
    pub files: usize,
    pub directories: usize,
    pub commits: usize,
    pub authors: usize,
    pub average_degree: f32,
    pub graph_density: f32,
    pub largest_component: usize,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityOverview {
    pub foreign_keys_passed: bool,
    pub missing_targets: usize,
    pub missing_sources: usize,
    pub orphans: usize,
    pub cycles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageOverview {
    pub git_history_enabled: bool,
    pub architecture_docs: usize,
    pub requirements: usize,
    pub ownership_enabled: bool,
    pub explicit_docs: usize,
    pub adrs: usize,
    pub decisions: usize,
    pub policies: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceOverview {
    pub why_exists_status: String,
    pub graph_status: String,
    pub git_memory_status: String,
    pub ownership_status: String,
    pub requirements_status: String,
    pub governance_status: String,
    pub impact_status: String,
    pub traceability_status: String,
    pub simulation_status: String,
    pub drift_status: String,
    pub last_query: Option<String>,
    pub last_query_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOverview {
    pub scanner_ms: u64,
    pub ast_parsing_ms: u64,
    pub git_memory_ms: u64,
    pub knowledge_graph_ms: u64,
    pub persistence_ms: u64,
    pub total_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub message: String,
    pub relative_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthOverview {
    pub score: i32,
    pub status: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionOverview {
    pub ares_version: String,
    pub schema_version: String,
    pub database_version: String,
    pub extension_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hit_rate: String,
    pub age: f32,
    pub ttl: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryDashboardResponse {
    pub schema_version: i32,
    pub generated_at: String,
    pub refreshing: bool,
    pub cache_stats: Option<CacheStats>,
    pub repository_id: String,
    pub repository: RepositoryOverview,
    pub graph: GraphOverview,
    pub integrity: IntegrityOverview,
    pub coverage: CoverageOverview,
    pub intelligence: IntelligenceOverview,
    pub performance: PerformanceOverview,
    pub health: HealthOverview,
    pub activity: Vec<ActivityEvent>,
    pub version: VersionOverview,
}
