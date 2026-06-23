use crate::id::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvolutionEventType {
    Created,
    Updated,
    OwnershipChanged,
    DriftDetected,
    StalenessDetected,
    Superseded,
    Deprecated,
    Restored,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Scanner,
    Test,
    RuntimeSignal,
    Human,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftEvidence {
    pub source_node: NodeId,
    pub observed_fact: String,
    pub confidence: f32,
    pub source: EvidenceSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolutionEvent {
    pub id: NodeId,
    pub target_node: NodeId,
    pub event_type: EvolutionEventType,
    /// Unix microseconds
    pub occurred_at: i64,
    pub actor: Option<String>,
    pub rationale: Option<String>,
    pub evidence_ids: Vec<NodeId>,
    pub confidence: f32,
}
