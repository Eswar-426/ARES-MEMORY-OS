use crate::models::{ComplianceResult, GovernanceCertification};
use crate::scorecard::calculate_scorecard;
use chrono::Utc;
use uuid::Uuid;

pub fn compute_certification(
    project_id: &str,
    results: &[ComplianceResult],
) -> GovernanceCertification {
    let scorecard = calculate_scorecard(results);

    let mut violations_count = 0;
    for res in results {
        violations_count += res.violations.len();
    }

    let policy_score = scorecard.overall_score;

    // Find the minimum category score
    let min_score = [
        scorecard.ownership_score,
        scorecard.traceability_score,
        scorecard.evidence_score,
        scorecard.approval_score,
        scorecard.retention_score,
        scorecard.security_score,
        scorecard.architecture_score,
    ]
    .into_iter()
    .fold(f32::MAX, f32::min);

    let level = if min_score >= 95.0 {
        crate::models::CertificationLevel::Platinum
    } else if min_score >= 90.0 {
        crate::models::CertificationLevel::Gold
    } else if min_score >= 75.0 {
        crate::models::CertificationLevel::Silver
    } else if min_score >= 50.0 {
        crate::models::CertificationLevel::Bronze
    } else {
        crate::models::CertificationLevel::None
    };

    let certified = level != crate::models::CertificationLevel::None;

    GovernanceCertification {
        id: Uuid::now_v7().to_string(),
        project_id: project_id.to_string(),
        certified,
        policy_score,
        level,
        violations_count,
        scorecard,
        evaluated_at: Utc::now().timestamp(),
    }
}
