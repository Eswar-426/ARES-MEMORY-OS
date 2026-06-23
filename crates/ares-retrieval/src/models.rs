use ares_core::types::node::GraphNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RankingWeights {
    pub authority: f32,
    pub traceability: f32,
    pub governance: f32,
    pub freshness: f32,
    pub completeness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub node: GraphNode,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryContextPack {
    pub requirement: Option<GraphNode>,
    pub decisions: Vec<GraphNode>,
    pub architecture: Vec<GraphNode>,
    pub code: Vec<GraphNode>,
    pub tests: Vec<GraphNode>,
    pub governance: GovernanceSummary,
    pub drift: Vec<GraphNode>,
    pub staleness: Vec<GraphNode>,
    pub completeness: CompletenessSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GovernanceSummary {
    pub explicit_owner: Option<String>,
    pub approvers: Vec<String>,
    pub missing_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletenessSummary {
    pub missing_links: Vec<String>,
    pub orphan_status: bool,
}
