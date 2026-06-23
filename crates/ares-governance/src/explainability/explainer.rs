use crate::models::{ComplianceViolation, GovernanceExplanation, GovernanceExplanationSummary};

pub struct GovernanceExplainer;

impl GovernanceExplainer {
    pub fn summarize(violation: &ComplianceViolation) -> GovernanceExplanationSummary {
        let (missing, owner, fix) = match violation.id.as_str() {
            "TRACE-001" => (
                "TracesTo edge".to_string(),
                "Architecture Team".to_string(),
                "Create TracesTo relationship linking requirement to implementation".to_string(),
            ),
            "TRACE-002" => (
                "TracesTo edge".to_string(),
                "Architecture Team".to_string(),
                "Create TracesTo relationship linking decision to implementation or context"
                    .to_string(),
            ),
            "OWN-001" => (
                "OwnedBy edge".to_string(),
                "Architecture Team".to_string(),
                "Create OwnedBy relationship linking node to a valid User or Team".to_string(),
            ),
            _ => (
                "Unknown".to_string(),
                "System".to_string(),
                "Review governance policy documentation".to_string(),
            ),
        };

        GovernanceExplanationSummary {
            id: violation.id.clone(),
            title: violation.policy_name.clone(),
            reason: violation.reason.clone(),
            missing,
            owner,
            fix,
        }
    }

    pub fn explain(violation: &ComplianceViolation) -> GovernanceExplanation {
        GovernanceExplanation {
            summary: Self::summarize(violation),
            evidence: violation.supporting_nodes.clone(),
            policy_id: violation.id.clone(),
            severity: violation.severity.clone(),
        }
    }
}
