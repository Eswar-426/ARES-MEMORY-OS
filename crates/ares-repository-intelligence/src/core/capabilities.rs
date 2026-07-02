use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    GraphSearch,
    WhyExists,
    ImpactAnalysis,
    Traceability,
    GitHistory,
    Requirements,
    Ownership,
    Simulation,
    Workspace,
    Knowledge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    pub provides: Vec<Capability>,
    pub requires: Vec<Capability>,
}
