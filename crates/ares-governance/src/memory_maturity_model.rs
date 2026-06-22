use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::coverage_engine::MemoryCoverageMetrics;
use crate::memory_debt_engine::{MemoryDebtMetrics, MemoryDebtLevel};
use crate::memory_health_engine::MemoryHealthScore;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
pub enum MemoryMaturityLevel {
    Level0Chaos,       // Repository Chaos
    Level1Documented,  // Documented
    Level2Traceable,   // Traceable
    Level3Governed,    // Governed
    Level4Measured,    // Measured
    Level5MemoryNative,// Memory Native
}

impl MemoryMaturityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryMaturityLevel::Level0Chaos => "Level 0: Repository Chaos",
            MemoryMaturityLevel::Level1Documented => "Level 1: Documented",
            MemoryMaturityLevel::Level2Traceable => "Level 2: Traceable",
            MemoryMaturityLevel::Level3Governed => "Level 3: Governed",
            MemoryMaturityLevel::Level4Measured => "Level 4: Measured",
            MemoryMaturityLevel::Level5MemoryNative => "Level 5: Memory Native",
        }
    }
}

pub struct MemoryMaturityEngine;

impl MemoryMaturityEngine {
    pub fn evaluate(
        coverage: &MemoryCoverageMetrics,
        debt: &MemoryDebtMetrics,
        health: &MemoryHealthScore,
        is_governance_enforced: bool,
    ) -> MemoryMaturityLevel {
        
        let has_requirements = coverage.requirements.total > 0;
        let has_traceability = coverage.requirements.covered > 0;
        let has_ownership = coverage.ownership.total > 0 && coverage.ownership.percentage > 0.0;
        let has_decisions = coverage.decisions.total > 0;
        let has_tests = coverage.tests.percentage > 0.0;
        
        // Ensure tracking is effectively taking place
        let is_measured = health.total_health > 0.0 && debt.total_debt_score < 2000; 

        if is_governance_enforced && is_measured && has_traceability && has_ownership {
            return MemoryMaturityLevel::Level5MemoryNative;
        }

        if is_measured && has_traceability && has_ownership {
            return MemoryMaturityLevel::Level4Measured;
        }

        if has_ownership && has_tests && has_decisions {
            return MemoryMaturityLevel::Level3Governed;
        }

        if has_traceability {
            return MemoryMaturityLevel::Level2Traceable;
        }

        if has_requirements {
            return MemoryMaturityLevel::Level1Documented;
        }

        MemoryMaturityLevel::Level0Chaos
    }
}
