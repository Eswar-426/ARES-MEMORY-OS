use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCertificationReport {
    pub canonical_questions_passed: usize,
    pub total_questions: usize,

    pub replay_safe: bool,
    pub graph_integrity_passed: bool,

    pub traceability_coverage: f64,
    pub decision_coverage: f64,
    pub evolution_coverage: f64,

    pub repository_health: f64,
    pub memory_health: f64,
    pub knowledge_debt: f64,

    pub policy_score: f64,
    pub governance_certified: bool,

    pub policy_drift: Option<ares_governance::models::PolicyDriftStatus>,
    pub enforcement: Option<ares_governance::models::EnforcementReadiness>,
    
    pub certification_level: ares_governance::models::CertificationLevel,

    pub certified: bool,
}
