use crate::models::{ComplianceViolation, EnforcementAction, MemoryRiskLevel};

pub struct RiskClassificationEngine;

impl RiskClassificationEngine {
    pub fn classify_risk(
        new_violations: &[ComplianceViolation],
        traceability_links_removed: usize,
        decisions_affected: usize,
        ownership_changes: usize,
    ) -> MemoryRiskLevel {
        let has_blocking_violations = new_violations
            .iter()
            .any(|v| v.enforcement == EnforcementAction::Block);

        if has_blocking_violations || traceability_links_removed > 5 {
            return MemoryRiskLevel::MemoryCritical;
        }

        if !new_violations.is_empty() || decisions_affected > 0 || ownership_changes > 2 {
            return MemoryRiskLevel::MemoryRisk;
        }

        if traceability_links_removed > 0 || ownership_changes > 0 {
            return MemoryRiskLevel::MemoryWarning;
        }

        MemoryRiskLevel::MemorySafe
    }
}
