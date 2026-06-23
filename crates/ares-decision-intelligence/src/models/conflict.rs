use ares_core::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    ContradictoryDecision,
    OverlappingScope,
    SupersededButActive,
    GovernanceConflict,
    AssumptionConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionConflict {
    pub source_decision_id: NodeId,
    pub target_decision_id: NodeId,
    pub conflict_type: ConflictType,
    pub rationale: String,
}
