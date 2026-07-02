use ares_core::types::node::{GraphEdge, GraphNode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPayload {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub nodes: usize,
    pub edges: usize,
    pub types: std::collections::HashMap<String, usize>,
}
use crate::core::response::Citation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeDetails {
    pub overview: NodeOverview,
    pub health: NodeHealth,
    pub ownership: NodeOwnership,
    pub relationships: NodeRelationships,
    pub history: NodeHistory,
    pub architecture: NodeArchitecture,
    pub analysis: NodeAnalysis,
    pub evidence: NodeEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOverview {
    pub node_id: String,
    pub node_type: String,
    pub language: Option<String>,
    pub repository: Option<String>,
    pub loc: Option<usize>,
    pub module: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub confidence: f32,
    pub owner_confidence: f32,
    pub last_modified_days_ago: Option<u32>,
    pub missing_requirements: bool,
    pub missing_decisions: bool,
    pub is_orphan: bool,
    pub drift: bool,
    pub test_coverage: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOwnership {
    pub primary_owner: Option<String>,
    pub git_authors: Vec<String>,
    pub team: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRelationships {
    pub calls: Vec<Citation>,
    pub called_by: Vec<Citation>,
    pub imports: Vec<Citation>,
    pub imported_by: Vec<Citation>,
    pub depends_on: Vec<Citation>,
    pub required_by: Vec<Citation>,
    pub implements: Vec<Citation>,
    pub inherited_by: Vec<Citation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHistory {
    pub created_at: Option<String>,
    pub last_modified: Option<String>,
    pub commits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeArchitecture {
    pub requirements: Vec<Citation>,
    pub adrs: Vec<Citation>,
    pub decisions: Vec<Citation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAnalysis {
    pub knowledge_debt: Option<String>,
    pub risk_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEvidence {
    pub files: Vec<Citation>,
    pub functions: Vec<Citation>,
    pub commits: Vec<Citation>,
}
