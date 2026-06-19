use serde::{Deserialize, Serialize};
use ares_traceability::{TraceabilityGraph, TraceTargetType};
use utoipa::ToSchema;
use crate::trace_analysis::TraceAnalysisEngine;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum ImpactCategory {
    Decision,
    Architecture,
    Code,
    Test,
    Runtime,
    Governance,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImpactBreakdown {
    pub category: ImpactCategory,
    pub affected_count: usize,
    pub weighted_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum ImpactSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum ChangeRisk {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RequirementImpactReport {
    pub requirement_id: String,
    pub blast_radius_score: f32,
    pub severity: ImpactSeverity,
    pub risk: ChangeRisk,
    pub affected_decisions: Vec<String>,
    pub affected_architecture: Vec<String>,
    pub affected_code: Vec<String>,
    pub affected_tests: Vec<String>,
    pub affected_runtime_metrics: Vec<String>,
    pub affected_governance: Vec<String>,
    pub impact_breakdown: Vec<ImpactBreakdown>,
}

pub struct RequirementImpactEngine<'a> {
    graph: &'a TraceabilityGraph,
}

impl<'a> RequirementImpactEngine<'a> {
    pub fn new(graph: &'a TraceabilityGraph) -> Self {
        Self { graph }
    }

    pub fn evaluate_impact(&self, req_id: &str) -> RequirementImpactReport {
        let resolver = TraceAnalysisEngine::new(&self.graph);
        
        let affected_decisions = resolver.get_downstream(req_id, TraceTargetType::Decision);
        let affected_architecture = resolver.get_downstream(req_id, TraceTargetType::Architecture);
        let affected_code = resolver.get_downstream(req_id, TraceTargetType::Code);
        let affected_tests = resolver.get_downstream(req_id, TraceTargetType::Test);
        let affected_runtime_metrics = resolver.get_downstream(req_id, TraceTargetType::RuntimeMetric);
        let affected_governance = resolver.get_downstream(req_id, TraceTargetType::Governance);
        
        let mut breakdowns = Vec::new();
        let mut total_score = 0.0;
        
        let decision_weight = 10.0;
        let architecture_weight = 8.0;
        let metric_weight = 7.0;
        let governance_weight = 6.0;
        let test_weight = 4.0;
        let code_weight = 2.0;
        
        if !affected_decisions.is_empty() {
            let score = affected_decisions.len() as f32 * decision_weight;
            total_score += score;
            breakdowns.push(ImpactBreakdown {
                category: ImpactCategory::Decision,
                affected_count: affected_decisions.len(),
                weighted_score: score,
            });
        }
        if !affected_architecture.is_empty() {
            let score = affected_architecture.len() as f32 * architecture_weight;
            total_score += score;
            breakdowns.push(ImpactBreakdown {
                category: ImpactCategory::Architecture,
                affected_count: affected_architecture.len(),
                weighted_score: score,
            });
        }
        if !affected_runtime_metrics.is_empty() {
            let score = affected_runtime_metrics.len() as f32 * metric_weight;
            total_score += score;
            breakdowns.push(ImpactBreakdown {
                category: ImpactCategory::Runtime,
                affected_count: affected_runtime_metrics.len(),
                weighted_score: score,
            });
        }
        if !affected_governance.is_empty() {
            let score = affected_governance.len() as f32 * governance_weight;
            total_score += score;
            breakdowns.push(ImpactBreakdown {
                category: ImpactCategory::Governance,
                affected_count: affected_governance.len(),
                weighted_score: score,
            });
        }
        if !affected_tests.is_empty() {
            let score = affected_tests.len() as f32 * test_weight;
            total_score += score;
            breakdowns.push(ImpactBreakdown {
                category: ImpactCategory::Test,
                affected_count: affected_tests.len(),
                weighted_score: score,
            });
        }
        if !affected_code.is_empty() {
            let score = affected_code.len() as f32 * code_weight;
            total_score += score;
            breakdowns.push(ImpactBreakdown {
                category: ImpactCategory::Code,
                affected_count: affected_code.len(),
                weighted_score: score,
            });
        }

        let blast_radius_score = total_score.min(100.0);
        
        let severity = match blast_radius_score {
            s if s >= 90.0 => ImpactSeverity::Critical,
            s if s >= 70.0 => ImpactSeverity::High,
            s if s >= 40.0 => ImpactSeverity::Medium,
            s if s > 0.0 => ImpactSeverity::Low,
            _ => ImpactSeverity::Low,
        };

        let mut risk_score = 0;
        if !affected_decisions.is_empty() { risk_score += 40; }
        if !affected_governance.is_empty() { risk_score += 30; }
        if !affected_architecture.is_empty() { risk_score += 20; }
        
        let risk = match risk_score {
            r if r >= 70 => ChangeRisk::Critical,
            r if r >= 40 => ChangeRisk::High,
            r if r >= 20 => ChangeRisk::Medium,
            r if r > 0 => ChangeRisk::Low,
            _ => ChangeRisk::Low,
        };

        RequirementImpactReport {
            requirement_id: req_id.to_string(),
            blast_radius_score,
            severity,
            risk,
            affected_decisions,
            affected_architecture,
            affected_code,
            affected_tests,
            affected_runtime_metrics,
            affected_governance,
            impact_breakdown: breakdowns,
        }
    }
}
