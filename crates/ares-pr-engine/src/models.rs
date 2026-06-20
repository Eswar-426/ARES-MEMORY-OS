use ares_governance::models::{ComplianceViolation, MemoryRiskLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDelta {
    pub added_nodes: usize,
    pub removed_nodes: usize,
    pub added_edges: usize,
    pub removed_edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryImpactReport {
    pub requirements_affected: usize,
    pub decisions_affected: usize,
    pub traceability_links_removed: usize,
    pub ownership_changes: usize,
    
    pub new_compliance_violations: usize,
    pub resolved_violations: usize,
    
    // Phase 10C Simulated Fields
    pub simulated_coverage_delta: f32,
    pub simulated_new_drift: usize,
    pub simulated_new_gaps: usize,
    
    pub new_violations_list: Vec<ComplianceViolation>,
    pub resolved_violations_list: Vec<ComplianceViolation>,
    
    pub risk_level: MemoryRiskLevel,
    pub baseline_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeReadiness {
    pub ready: bool,
    pub blocking_violations: Vec<ComplianceViolation>,
    pub warnings: Vec<String>,
    pub impact: MemoryImpactReport,
    pub graph_delta: GraphDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub graph: ares_knowledge_graph::models::KnowledgeGraph,
    pub compliance: Vec<ares_governance::models::ComplianceResult>,
    pub scorecard: ares_governance::models::GovernanceScorecard,
}
