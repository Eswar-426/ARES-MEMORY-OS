use ares_core::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub node_id: NodeId,
    pub direct_dependents: usize,
    pub indirect_dependents: usize,
    pub affected_files: usize,
    pub impact_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeCandidate {
    pub node_id: NodeId,
    pub label: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub nodes: Vec<NodeId>,
    pub cycle_length: usize,
    pub severity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub node_id: NodeId,
    pub degree: usize,
    pub in_degree: usize,
    pub out_degree: usize,
    pub betweenness: f64,
    pub risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskReport {
    pub file: String,
    pub risk_score: f64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHealth {
    pub score: f64,
}
