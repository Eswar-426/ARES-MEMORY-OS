use crate::models::{HealthGainBreakdown, ResolutionCategory, ResolutionTemplate};
use ares_gap_engine::models::GapType;

pub struct MemoryHealthSimulator;

impl MemoryHealthSimulator {
    pub fn new() -> Self {
        Self
    }

    pub fn simulate(
        &self,
        gap_type: &GapType,
        template: &ResolutionTemplate,
    ) -> (f64, f64, HealthGainBreakdown) {
        let health_gain = template.expected_health_gain;
        let debt_reduction = template.expected_debt_reduction;

        // Breakdown based on category (we'll infer categories based on the gap type)
        let mut breakdown = HealthGainBreakdown {
            ownership_gain: 0.0,
            traceability_gain: 0.0,
            governance_gain: 0.0,
            validation_gain: 0.0,
        };

        match gap_type {
            GapType::MissingDecision | GapType::MissingImplementation | GapType::OrphanCode => {
                breakdown.traceability_gain = health_gain * 0.8;
                breakdown.governance_gain = health_gain * 0.2;
            }
            GapType::MissingOwner => {
                breakdown.ownership_gain = health_gain;
            }
            GapType::MissingEvidence => {
                breakdown.governance_gain = health_gain * 0.8;
                breakdown.validation_gain = health_gain * 0.2;
            }
            GapType::StaleRequirement => {
                breakdown.governance_gain = health_gain * 0.5;
                breakdown.traceability_gain = health_gain * 0.5;
            }
        }

        (health_gain, debt_reduction, breakdown)
    }

    pub fn infer_category(&self, gap_type: &GapType) -> ResolutionCategory {
        match gap_type {
            GapType::MissingOwner => ResolutionCategory::Ownership,
            GapType::MissingDecision | GapType::MissingImplementation | GapType::OrphanCode => {
                ResolutionCategory::Traceability
            }
            GapType::MissingEvidence => ResolutionCategory::Governance,
            GapType::StaleRequirement => ResolutionCategory::Documentation,
        }
    }
}

impl Default for MemoryHealthSimulator {
    fn default() -> Self {
        Self::new()
    }
}
