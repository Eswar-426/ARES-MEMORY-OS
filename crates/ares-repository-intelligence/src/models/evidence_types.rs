use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Typed evidence primitives — used by EngineeringEvidence
// ═══════════════════════════════════════════════════════════════════

/// A typed graph relationship (incoming or outgoing dependency).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DependencyEvidence {
    pub id: String,
    pub label: String,
    pub edge_type: String,
    pub node_type: String,
    pub file_path: Option<String>,
}

/// A lightweight reference to any entity in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRef {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub file_path: Option<String>,
}

/// A code dependency with relationship context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRef {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub file_path: Option<String>,
    pub relationship: String,
    pub is_test: bool,
}

/// A contributor with activity context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorRef {
    pub name: String,
    pub commit_count: usize,
    pub is_primary: bool,
}

/// A single git commit relevant to an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitEvidence {
    pub hash: String,
    pub message: String,
    pub date: String,
    pub author: String,
}

/// Quantitative metrics for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetrics {
    pub lines_of_code: Option<usize>,
    pub complexity: Option<f32>,
    pub test_coverage: Option<f32>,
    pub dependency_count: usize,
    pub dependent_count: usize,
}

/// Temporal metadata for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamps {
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub last_committed: Option<String>,
}

/// Risk classification for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

