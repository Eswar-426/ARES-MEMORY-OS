pub mod engines;
pub mod models;

// Legacy shims for compatibility with other crates
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionSummary {
    pub id: String,
    pub title: String,
    pub status: DecisionStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DecisionStatus {
    Proposed,
    Approved,
    Rejected,
    Deprecated,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionCoverage {
    pub percentage: f32,
}

pub mod storage {
    pub struct DecisionStore;
    pub struct DecisionEdgeProvider;
}

pub mod health {
    pub struct DecisionHealthEngine;
}
