use serde::{Serialize, Deserialize};

// ─────────────────────────────────────────────────────────────────
// Trace Status — Memory gaps are first-class information
// ─────────────────────────────────────────────────────────────────

/// Reasoning must never panic because memory is incomplete.
/// TraceStatus communicates the completeness of a traversal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceStatus {
    /// Full chain discovered: Requirement → Decision → Architecture → Code
    Complete,
    /// Some links found but chain is incomplete (e.g. missing Requirement)
    Partial,
    /// Target node has no upstream connections at all
    Orphaned,
    /// Structural gap detected in the hierarchy (e.g. Decision exists but no Architecture below it)
    GapDetected,
}

impl std::fmt::Display for TraceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceStatus::Complete => write!(f, "Complete"),
            TraceStatus::Partial => write!(f, "Partial"),
            TraceStatus::Orphaned => write!(f, "Orphaned"),
            TraceStatus::GapDetected => write!(f, "GapDetected"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Missing Memory — What ARES expected but did not find
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingMemory {
    /// The node that has a gap above it
    pub node_id: String,
    /// The node type that was expected as a parent
    pub expected_type: String,
    /// The actual parent found (if any)
    pub actual_parent: Option<String>,
}

impl std::fmt::Display for MissingMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} missing upstream {}", self.node_id, self.expected_type)
    }
}

// ─────────────────────────────────────────────────────────────────
// Memory Gap — Structural breaks in the hierarchy
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGap {
    /// The type that should have an edge to `to_type`
    pub from_type: String,
    /// The type that is missing a parent
    pub to_type: String,
    /// The specific node ID where the gap was detected
    pub node_id: String,
    /// Description of the gap
    pub gap_description: String,
    /// Confidence that this is a real gap (1.0 = certain)
    pub confidence: f32,
}

// ─────────────────────────────────────────────────────────────────
// Trace Result — Unified return type from PathEngine
// ─────────────────────────────────────────────────────────────────

use ares_core::types::node::GraphNode;

#[derive(Debug, Clone)]
pub struct TraceResult {
    /// The nodes discovered during traversal
    pub nodes: Vec<GraphNode>,
    /// Completeness status of the traversal
    pub status: TraceStatus,
    /// Memory gaps detected during traversal
    pub missing: Vec<MissingMemory>,
    /// The path taken (node labels in order)
    pub path: Vec<String>,
    /// Hop distances for each node (node_id -> distance from start)
    pub distances: Vec<(String, usize)>,
    /// Number of nodes visited during traversal
    pub nodes_visited: usize,
    /// Number of edges traversed during traversal
    pub edges_visited: usize,
    /// Maximum depth reached
    pub max_depth: usize,
    /// Number of database queries made
    pub query_count: usize,
}

// ─────────────────────────────────────────────────────────────────
// Why Report — Explainability-first lineage report
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhyReport {
    pub target_id: String,
    pub requirements: Vec<String>,
    pub decisions: Vec<String>,
    pub architectures: Vec<String>,
    /// The source that discovered this lineage (e.g. "agent", "scanner")
    pub source: String,
    /// Evidence trail
    pub evidence: Vec<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// The traversal path taken (node labels in order)
    pub path: Vec<String>,
    /// Trace completeness
    pub status: TraceStatus,
    /// Missing memory detected during traversal
    pub missing: Vec<MissingMemory>,
}

// ─────────────────────────────────────────────────────────────────
// Impact Report — Multi-hop classification
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub affected_requirements: Vec<String>,
    pub affected_decisions: Vec<String>,
    pub affected_architecture: Vec<String>,
    pub affected_files: Vec<String>,
    pub risk_score: f32,
    /// Multi-hop reachability classifications: (node_label, reachability)
    pub classifications: Vec<(String, Reachability)>,
    
    // Traversal Metrics
    pub nodes_visited: usize,
    pub edges_visited: usize,
    pub max_depth: usize,
    pub query_count: usize,
}

// ─────────────────────────────────────────────────────────────────
// Breakage Report
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakageReport {
    pub impacted_files: Vec<String>,
    pub impacted_tests: Vec<String>,
    pub impacted_runtime_signals: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────
// Reachability — Multi-hop impact classification
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reachability {
    /// Distance 1 from changed node
    Direct,
    /// Distance 2 from changed node
    Indirect,
    /// Distance >= 3 from changed node
    Transitive,
}

impl Reachability {
    pub fn from_distance(d: usize) -> Self {
        match d {
            0 | 1 => Reachability::Direct,
            2 => Reachability::Indirect,
            _ => Reachability::Transitive,
        }
    }
}

impl std::fmt::Display for Reachability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reachability::Direct => write!(f, "Direct"),
            Reachability::Indirect => write!(f, "Indirect"),
            Reachability::Transitive => write!(f, "Transitive"),
        }
    }
}
