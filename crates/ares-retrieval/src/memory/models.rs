use serde::{Deserialize, Serialize};
use ares_requirements::models::Requirement;
use ares_decision_intelligence::models::{Decision, DecisionEvidence, DecisionOutcome};
use ares_gap_engine::models::{Gap, KnowledgeDebt, RepositoryHealthReport};
use ares_resolution_engine::models::ResolutionPlan;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryIntent {
    Why,
    What,
    Who,
    When,
    Impact,
    Traceability,
    Governance,
    Debt,
    Health,
    Resolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryQuery {
    pub query: String,
    pub intent: QueryIntent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryPattern {
    pub intent: QueryIntent,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrievalStrategy {
    RequirementFocused,
    DecisionFocused,
    TraceabilityFocused,
    GovernanceFocused,
    DebtFocused,
    HealthFocused,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TracePath {
    pub nodes: Vec<String>,
    pub relationships: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextQuality {
    pub traceability_score: f64,
    pub health_score: f64,
    pub debt_score: f64,
    pub completeness_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContextPackage {
    pub requirements: Vec<Requirement>,
    pub decisions: Vec<Decision>,
    pub evidence: Vec<DecisionEvidence>,
    pub outcomes: Vec<DecisionOutcome>,
    pub gaps: Vec<Gap>,
    pub resolution_plans: Vec<ResolutionPlan>,
    pub health_report: Option<RepositoryHealthReport>,
    pub knowledge_debt: Option<KnowledgeDebt>,
    pub trace_paths: Vec<TracePath>,
    pub context_quality: Option<ContextQuality>, // Quality is injected at the end
}

impl MemoryContextPackage {
    pub fn new() -> Self {
        Self {
            requirements: vec![],
            decisions: vec![],
            evidence: vec![],
            outcomes: vec![],
            gaps: vec![],
            resolution_plans: vec![],
            health_report: None,
            knowledge_debt: None,
            trace_paths: vec![],
            context_quality: None,
        }
    }
}

impl Default for MemoryContextPackage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetrievalCoverage {
    pub requirements_found: usize,
    pub decisions_found: usize,
    pub evidence_found: usize,
    pub gaps_found: usize,
    pub resolutions_found: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalExplanationNode {
    pub source: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryExplanation {
    pub why_this_was_returned: String,
    pub confidence: f64,
    pub retrieval_strategy: RetrievalStrategy,
    pub tree: Vec<RetrievalExplanationNode>,
    pub coverage: RetrievalCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResult {
    pub query: MemoryQuery,
    pub context: MemoryContextPackage,
    pub explanation: MemoryExplanation,
}
