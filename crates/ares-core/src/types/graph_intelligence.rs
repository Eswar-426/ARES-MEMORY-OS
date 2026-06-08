use crate::{GraphEdge, GraphNode, NodeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisScope {
    Project,
    Workspace,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub density: f64,
    pub average_degree: f64,
    pub connected_components: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub statistics: GraphStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactPrediction {
    pub target_node: NodeId,
    pub blast_radius: usize,
    pub risk_score: f64,
    pub affected_modules: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStep {
    pub description: String,
    pub node_id: Option<NodeId>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub target_node: NodeId,
    pub root_cause_node: Option<NodeId>,
    pub evidence_chain: Vec<EvidenceStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureHealthReport {
    pub fan_in_hotspots: Vec<NodeId>,
    pub fan_out_hotspots: Vec<NodeId>,
    pub unstable_modules: Vec<NodeId>,
    pub orphan_modules: Vec<NodeId>,
    pub dependency_bottlenecks: Vec<NodeId>,
    pub cycles: Vec<Vec<NodeId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCluster {
    pub id: String,
    pub name: String,
    pub nodes: Vec<NodeId>,
    pub cohesion: f64,
    pub coupling: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: f64,
    pub dependency_risk: f64,
    pub volatility_risk: f64,
    pub architectural_debt_risk: f64,
    pub knowledge_risk: f64,
}
