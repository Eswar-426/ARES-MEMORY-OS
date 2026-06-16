use crate::models::{
    EffortLevel, ResolutionAction, ResolutionConfidence, ResolutionImpact, ResolutionPlan,
    ResolutionReport,
};
use crate::prioritizer::ResolutionPrioritizer;
use crate::rules::ResolutionRuleEngine;
use crate::simulator::MemoryHealthSimulator;
use ares_gap_engine::models::{GapSeverity, RepositoryHealthReport};
use chrono::Utc;
use uuid::Uuid;

pub struct ResolutionEngine {
    rule_engine: ResolutionRuleEngine,
    prioritizer: ResolutionPrioritizer,
    simulator: MemoryHealthSimulator,
}

impl ResolutionEngine {
    pub fn new() -> Self {
        Self {
            rule_engine: ResolutionRuleEngine::new(),
            prioritizer: ResolutionPrioritizer::new(),
            simulator: MemoryHealthSimulator::new(),
        }
    }

    pub fn generate_report(&self, health_report: &RepositoryHealthReport) -> ResolutionReport {
        let mut recommended_plans = Vec::new();

        for gap in &health_report.gaps {
            if let Some(reason) = &gap.reason {
                let template = self.rule_engine.get_template(&gap.gap_type, &reason.root_cause);

                let category = self.simulator.infer_category(&gap.gap_type);
                let (health_gain, debt_reduction, breakdown) =
                    self.simulator.simulate(&gap.gap_type, &template);
                let priority = self
                    .prioritizer
                    .prioritize(gap, &health_report.health, &health_report.knowledge_debt);

                let actions: Vec<ResolutionAction> = template
                    .actions
                    .into_iter()
                    .map(|action_type| ResolutionAction {
                        id: Uuid::now_v7().to_string(),
                        title: format!("{:?}", action_type),
                        description: format!("Execute action: {:?}", action_type),
                        action_type,
                        target_entities: vec![gap.source_id.clone()],
                        expected_impact: ResolutionImpact {
                            entities_resolved: 1,
                            severity_reduction: match gap.severity {
                                GapSeverity::Critical => 30.0,
                                GapSeverity::Warning => 15.0,
                                GapSeverity::Info => 5.0,
                            },
                        },
                    })
                    .collect();

                // Determine execution confidence based on action types and severity
                let confidence = if actions
                    .iter()
                    .any(|a| a.title.contains("Review") || a.title.contains("Architecture"))
                {
                    ResolutionConfidence::Medium
                } else {
                    ResolutionConfidence::Guaranteed
                };

                let plan = ResolutionPlan {
                    id: Uuid::now_v7().to_string(),
                    gap_id: gap.id.clone(),
                    root_cause: reason.root_cause.clone(),
                    category,
                    confidence,
                    actions,
                    priority,
                    estimated_effort: EffortLevel::Medium,
                    expected_health_gain: health_gain,
                    expected_debt_reduction: debt_reduction,
                    health_gain_breakdown: breakdown,
                    generated_at: Utc::now().timestamp(),
                };

                recommended_plans.push(plan);
            }
        }

        // Sort plans by priority (Critical first)
        recommended_plans.sort_by(|a, b| b.priority.cmp(&a.priority));

        ResolutionReport {
            repository_health: health_report.health.overall_score,
            knowledge_debt: health_report.knowledge_debt.debt_score,
            critical_gaps: health_report.knowledge_debt.critical_gaps,
            recommended_plans,
        }
    }
}

impl Default for ResolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}
