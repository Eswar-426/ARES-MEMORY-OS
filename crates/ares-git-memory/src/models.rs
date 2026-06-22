use serde::{Deserialize, Serialize};
use ares_core::{GraphNode, GraphEdge};

/// How the memory fact was captured.
/// Directly maps to ARCH-MEMORY-CAPTURE.md Section 6.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CaptureMethod {
    /// Human-authored artifacts with clear intent (CODEOWNERS, ADRs)
    Explicit,
    /// Machine-readable facts from the repository (git log, git tag)
    Repository,
    /// Classified from repository data (conventional commit types)
    Inferred,
    /// Statistical attribution (git blame line ownership)
    Heuristic,
}

impl CaptureMethod {
    pub fn base_confidence(&self) -> f64 {
        match self {
            CaptureMethod::Explicit => 1.0,
            CaptureMethod::Repository => 0.8,
            CaptureMethod::Inferred => 0.6,
            CaptureMethod::Heuristic => 0.4,
        }
    }
}

/// Provenance metadata attached to every captured fact.
/// Stored in the `properties` JSON of GraphNode/GraphEdge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceProvenance {
    pub source_system: String,
    pub source_id: String,
    pub capture_method: CaptureMethod,
    pub captured_at: i64,
    pub confidence: f64,
}

/// A memory source discovered in a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySource {
    pub name: String,
    pub tier: SourceTier,
    pub available: bool,
    pub captured: bool,
    pub node_count: u64,
    pub edge_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceTier {
    Explicit,
    Repository,
    External,
}

/// Result of git memory extraction — nodes and edges to upsert.
pub struct GitMemoryResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub sources: Vec<MemorySource>,
}
