use crate::models::{Gap, GapSeverity, ImpactRadius, PriorityScore};
use ares_core::AresError;
use ares_decision_intelligence::storage::DecisionEdgeProvider;
use ares_requirements::storage::RequirementStore;
use ares_store::Store;
use std::sync::Arc;

pub struct GapPrioritizer {
    store: Arc<Store>,
}

impl GapPrioritizer {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Evaluates a list of reasoned gaps and attaches ImpactRadius and PriorityScore.
    pub fn prioritize(&self, mut gaps: Vec<Gap>) -> Result<Vec<Gap>, AresError> {
        let _req_store = RequirementStore::new((*self.store).clone());
        let _dec_edges = DecisionEdgeProvider::new((*self.store).clone());

        for gap in &mut gaps {
            let mut req_impact = 0;
            let mut dec_impact = 0;
            let mut arch_impact = 0;
            let mut code_impact = 0;

            // Simple heuristic mapping for MVP
            // Ideally this queries `ares-traceability` and performs a BFS to count all downstream nodes.
            match gap.gap_type {
                crate::models::GapType::MissingDecision => {
                    req_impact = 1;
                }
                crate::models::GapType::MissingEvidence | crate::models::GapType::MissingOwner => {
                    dec_impact = 1;

                    // A decision missing evidence might affect downstream requirements if they trace to it.
                    // For now, we simulate radius
                    req_impact = 2;
                    arch_impact = 1;
                }
                crate::models::GapType::MissingImplementation => {
                    req_impact = 1;
                    dec_impact = 1;
                    code_impact = 10;
                }
                crate::models::GapType::StaleRequirement => {
                    req_impact = 1;
                }
                crate::models::GapType::OrphanCode => {
                    code_impact = 50;
                }
            }

            let impact = ImpactRadius {
                requirements: req_impact,
                decisions: dec_impact,
                architecture_components: arch_impact,
                code_artifacts: code_impact,
            };

            // Calculate score (0-100)
            let base_score = match gap.severity {
                GapSeverity::Info => 10.0,
                GapSeverity::Warning => 40.0,
                GapSeverity::Critical => 80.0,
            };

            let radius_multiplier = 1.0
                + ((req_impact + dec_impact + arch_impact) as f64 * 0.05)
                + (code_impact as f64 * 0.01);
            let raw_score = base_score * radius_multiplier;
            let score = raw_score.min(100.0);

            let criticality = if score >= 80.0 {
                "Critical".to_string()
            } else if score >= 50.0 {
                "High".to_string()
            } else if score >= 30.0 {
                "Medium".to_string()
            } else {
                "Low".to_string()
            };

            gap.impact_radius = Some(impact);
            gap.priority_score = Some(PriorityScore { score, criticality });
        }

        // Sort gaps by priority score descending
        gaps.sort_by(|a, b| {
            let score_a = a.priority_score.as_ref().map(|p| p.score).unwrap_or(0.0);
            let score_b = b.priority_score.as_ref().map(|p| p.score).unwrap_or(0.0);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(gaps)
    }
}
