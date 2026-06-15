use serde::{Deserialize, Serialize};
use ares_core::GraphNode;
use super::{ContextMetrics, DependencyTrace, ImpactReport};
use crate::traversal::ArchitecturePath;
use crate::query::intent::QueryIntent;
use chrono::{DateTime, Utc};

/// Tracks the rationale behind why nodes were selected
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetrievalExplanation {
    pub selected_nodes: Vec<String>,
    pub ranking_reasons: Vec<String>,
}

/// Limits to strictly enforce context constraints before passing to LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudget {
    pub max_files: usize,
    pub max_nodes: usize,
    pub max_dependencies: usize,
    pub max_impact_entries: usize,
    pub max_snippets: usize,
    pub max_characters: usize,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_files: 20,
            max_nodes: 100,
            max_dependencies: 50,
            max_impact_entries: 50,
            max_snippets: 20,
            max_characters: 100_000,
        }
    }
}

/// A ContextPack is the exact, final payload sent to agents.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextPack {
    pub query: String,
    pub intent: QueryIntent,

    pub summary: String,

    pub relevant_files: Vec<String>,
    pub relevant_nodes: Vec<GraphNode>,

    pub dependency_trace: Vec<DependencyTrace>,
    pub impact_analysis: Vec<ImpactReport>,
    pub architecture_paths: Vec<ArchitecturePath>,

    pub memory_snippets: Vec<String>,

    pub confidence_score: f32,
    pub generated_at: DateTime<Utc>,
    pub retrieval_time_ms: u64,
    
    pub retrieval_explanation: RetrievalExplanation,
    pub metrics: ContextMetrics,
}
