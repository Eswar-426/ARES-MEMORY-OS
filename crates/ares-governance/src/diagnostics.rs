use crate::models::ComplianceViolation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub compliant: bool,
    pub violations: Vec<ComplianceViolation>,
}

pub fn why_is_this_non_compliant(violations: &[ComplianceViolation]) -> DiagnosticsReport {
    DiagnosticsReport {
        compliant: violations.is_empty(),
        violations: violations.to_vec(),
    }
}
