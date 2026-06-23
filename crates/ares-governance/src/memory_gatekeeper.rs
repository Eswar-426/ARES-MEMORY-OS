use serde::{Deserialize, Serialize};

use crate::coverage_engine::MemoryCoverageMetrics;
use crate::memory_debt_engine::MemoryDebtMetrics;
use crate::memory_health_engine::MemoryHealthScore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatekeeperStatus {
    Pass,
    SoftFail(Vec<String>),
    HardFail(Vec<String>),
}

pub struct MemoryGatekeeper;

impl MemoryGatekeeper {
    pub fn evaluate_delta(
        before_coverage: &MemoryCoverageMetrics,
        after_coverage: &MemoryCoverageMetrics,
        before_debt: &MemoryDebtMetrics,
        after_debt: &MemoryDebtMetrics,
        before_health: &MemoryHealthScore,
        after_health: &MemoryHealthScore,
    ) -> GatekeeperStatus {
        let mut soft_fail_reasons = Vec::new();
        let mut hard_fail_reasons = Vec::new();

        // Check Hard Fails (Critical Removals)
        if after_coverage.requirements.total < before_coverage.requirements.total {
            hard_fail_reasons.push("Requirement removed".to_string());
        }
        if after_coverage.decisions.total < before_coverage.decisions.total {
            hard_fail_reasons.push("Decision removed".to_string());
        }
        if after_coverage.ownership.covered < before_coverage.ownership.covered {
            hard_fail_reasons.push("Owner removed".to_string());
        }

        // Check Percentage Deltas
        let coverage_delta = after_coverage.overall.percentage - before_coverage.overall.percentage;
        let health_delta = after_health.total_health - before_health.total_health;

        let debt_delta_pct = if before_debt.total_debt_score > 0 {
            ((after_debt.total_debt_score as f64 - before_debt.total_debt_score as f64)
                / before_debt.total_debt_score as f64)
                * 100.0
        } else if after_debt.total_debt_score > 0 {
            100.0
        } else {
            0.0
        };

        // Hard Fail Thresholds
        if coverage_delta < -5.0 {
            hard_fail_reasons.push(format!(
                "Coverage regressed by {:.1}% (Threshold: -5%)",
                coverage_delta
            ));
        }
        if health_delta < -10.0 {
            hard_fail_reasons.push(format!(
                "Health regressed by {:.1}% (Threshold: -10%)",
                health_delta
            ));
        }
        if debt_delta_pct > 25.0 {
            hard_fail_reasons.push(format!(
                "Debt increased by {:.1}% (Threshold: +25%)",
                debt_delta_pct
            ));
        }

        if !hard_fail_reasons.is_empty() {
            return GatekeeperStatus::HardFail(hard_fail_reasons);
        }

        // Soft Fail Thresholds
        if coverage_delta < -1.5 {
            soft_fail_reasons.push(format!(
                "Coverage regressed by {:.1}% (Warning threshold: -1.5%)",
                coverage_delta
            ));
        }
        if health_delta < -5.0 {
            soft_fail_reasons.push(format!(
                "Health regressed by {:.1}% (Warning threshold: -5%)",
                health_delta
            ));
        }
        if debt_delta_pct > 5.0 {
            soft_fail_reasons.push(format!(
                "Debt increased by {:.1}% (Warning threshold: +5%)",
                debt_delta_pct
            ));
        }

        if !soft_fail_reasons.is_empty() {
            return GatekeeperStatus::SoftFail(soft_fail_reasons);
        }

        GatekeeperStatus::Pass
    }
}
