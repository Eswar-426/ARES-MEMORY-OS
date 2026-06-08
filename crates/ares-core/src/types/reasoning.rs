use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReasoningEvidence {
    pub memory_ids: Vec<String>,
    pub decision_ids: Vec<String>,
    pub contradiction_ids: Vec<String>,
    pub graph_paths: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningGraphNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningGraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReasoningGraph {
    pub nodes: Vec<ReasoningGraphNode>,
    pub edges: Vec<ReasoningGraphEdge>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimelineEventType {
    Created,
    Updated,
    Superseded,
    Contradicted,
    Deprecated,
    Reactivated,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeGap {
    pub missing_dependencies: Vec<String>,
    pub missing_decisions: Vec<String>,
    pub low_confidence_areas: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReasoningDiagnostics {
    pub retrieval_ms: u64,
    pub dependency_ms: u64,
    pub contradiction_ms: u64,
    pub evolution_ms: u64,
    pub assembly_ms: u64,
    pub total_ms: u64,
}
