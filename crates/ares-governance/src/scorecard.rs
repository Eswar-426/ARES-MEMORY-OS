use crate::models::{ComplianceResult, GovernanceScorecard, PolicyCategory};

pub fn calculate_scorecard(results: &[ComplianceResult]) -> GovernanceScorecard {
    let checks_run = results.len();

    if results.is_empty() {
        return GovernanceScorecard {
            ownership_score: 0.0,
            traceability_score: 0.0,
            evidence_score: 0.0,
            approval_score: 0.0,
            retention_score: 0.0,
            security_score: 0.0,
            architecture_score: 0.0,
            overall_score: 0.0,
            compliance_checks_run: 0,
        };
    }

    let mut scores = std::collections::HashMap::new();
    let mut counts = std::collections::HashMap::new();

    for res in results {
        let is_compliant = if res.compliant { 100.0 } else { 0.0 };

        let total = scores.entry(res.category.clone()).or_insert(0.0);
        let count = counts.entry(res.category.clone()).or_insert(0);

        *total += is_compliant;
        *count += 1;
    }

    let get_score = |category: PolicyCategory| -> f32 {
        let count = *counts.get(&category).unwrap_or(&0);
        if count > 0 {
            *scores.get(&category).unwrap_or(&0.0) / count as f32
        } else {
            100.0
        }
    };

    let ownership_score = get_score(PolicyCategory::Ownership);
    let traceability_score = get_score(PolicyCategory::Traceability);
    let evidence_score = get_score(PolicyCategory::Evidence);
    let approval_score = get_score(PolicyCategory::Approval);
    let retention_score = get_score(PolicyCategory::Retention);
    let security_score = get_score(PolicyCategory::Security);
    let architecture_score = get_score(PolicyCategory::Architecture);

    let overall_score = (ownership_score
        + traceability_score
        + evidence_score
        + approval_score
        + retention_score
        + security_score
        + architecture_score)
        / 7.0;

    GovernanceScorecard {
        ownership_score,
        traceability_score,
        evidence_score,
        approval_score,
        retention_score,
        security_score,
        architecture_score,
        overall_score,
        compliance_checks_run: checks_run,
    }
}
