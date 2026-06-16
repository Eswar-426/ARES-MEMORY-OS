use serde::{Deserialize, Serialize};
use ares_core::GraphNode;
use super::{ImpactReport, DependencyTrace, FileExplanation, RepositoryInsight};

/// A ContextBundle aggregates various context objects for consumption.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextBundle {
    pub query: String,
    pub intent: crate::query::intent::QueryIntent,
    pub target_nodes: Vec<GraphNode>,
    pub ranked_nodes: Vec<GraphNode>,
    pub impact_reports: Vec<ImpactReport>,
    pub dependency_traces: Vec<DependencyTrace>,
    pub explanations: Vec<FileExplanation>,
    pub repository_insights: Vec<RepositoryInsight>,
    pub metrics: crate::models::metrics::ContextMetrics,
    pub reachable_nodes: usize,
}
