use crate::id::NodeId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    ScannerFact,
    TestFact,
    RuntimeFact,
    OwnershipFact,
    DependencyFact,
    ConfigurationFact,
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Scanner,
    TestRunner,
    RuntimeSignal,
    Human,
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: NodeId,
    pub evidence_type: EvidenceType,
    pub source_node: NodeId,
    pub observed_value: String,
    pub observed_at: DateTime<Utc>,
    pub confidence: f32,
    pub source: EvidenceSource,
}
