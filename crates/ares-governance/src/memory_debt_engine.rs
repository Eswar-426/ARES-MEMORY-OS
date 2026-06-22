use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::coverage_engine::MemoryCoverageMetrics;
use crate::memory_drift_engine::MemoryDriftMetrics;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum MemoryDebtLevel {
    Healthy,  // 0-100
    Moderate, // 101-500
    High,     // 501-2000
    Critical, // 2000+
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryDebtMetrics {
    pub missing_requirements_penalty: u64,
    pub missing_decisions_penalty: u64,
    pub missing_owners_penalty: u64,
    pub missing_evidence_penalty: u64,
    pub missing_tests_penalty: u64,
    pub drift_penalty: u64,
    pub total_debt_score: u64,
    pub severity: MemoryDebtLevel,
}

pub struct MemoryDebtEngine;

impl MemoryDebtEngine {
    pub fn calculate(coverage: &MemoryCoverageMetrics, drift: &MemoryDriftMetrics) -> MemoryDebtMetrics {
        // Point values as per architectural specification
        const REQ_WEIGHT: u64 = 10;
        const DEC_WEIGHT: u64 = 8;
        const OWN_WEIGHT: u64 = 7;
        const EVD_WEIGHT: u64 = 5;
        const TST_WEIGHT: u64 = 3;
        const DRIFT_WEIGHT: u64 = 2;

        let missing_requirements = coverage.requirements.total.saturating_sub(coverage.requirements.covered);
        let missing_decisions = coverage.decisions.total.saturating_sub(coverage.decisions.covered);
        let missing_owners = coverage.ownership.total.saturating_sub(coverage.ownership.covered);
        let missing_evidence = coverage.evidence.total.saturating_sub(coverage.evidence.covered);
        let missing_tests = coverage.tests.total.saturating_sub(coverage.tests.covered);
        let drifted_artifacts = drift.artifacts_changed_without_memory_updates;

        let missing_requirements_penalty = missing_requirements * REQ_WEIGHT;
        let missing_decisions_penalty = missing_decisions * DEC_WEIGHT;
        let missing_owners_penalty = missing_owners * OWN_WEIGHT;
        let missing_evidence_penalty = missing_evidence * EVD_WEIGHT;
        let missing_tests_penalty = missing_tests * TST_WEIGHT;
        let drift_penalty = drifted_artifacts * DRIFT_WEIGHT;

        let total_debt_score = missing_requirements_penalty
            + missing_decisions_penalty
            + missing_owners_penalty
            + missing_evidence_penalty
            + missing_tests_penalty
            + drift_penalty;

        let severity = match total_debt_score {
            0..=100 => MemoryDebtLevel::Healthy,
            101..=500 => MemoryDebtLevel::Moderate,
            501..=2000 => MemoryDebtLevel::High,
            _ => MemoryDebtLevel::Critical,
        };

        MemoryDebtMetrics {
            missing_requirements_penalty,
            missing_decisions_penalty,
            missing_owners_penalty,
            missing_evidence_penalty,
            missing_tests_penalty,
            drift_penalty,
            total_debt_score,
            severity,
        }
    }
}
