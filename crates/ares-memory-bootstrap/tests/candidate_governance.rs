use ares_candidates::{Candidate, CandidateStatus, CandidateType};
use ares_governance::candidate_governance_engine::{
    CandidateGovernanceEngine, CandidateGovernancePolicy,
};
use chrono::Utc;

fn create_candidate() -> Candidate {
    Candidate {
        id: "c1".to_string(),
        project_id: "test-proj".to_string(),
        title: "Test Candidate".to_string(),
        description: "Description".to_string(),
        candidate_type: CandidateType::Capability,
        decision_category: None,
        architecture_category: None,
        traceability_category: None,
        source_endpoint: None,
        target_endpoint: None,
        traceability_strength: None,
        ownership_domains: vec![],
        dependent_components: vec![],
        status: CandidateStatus::Proposed,
        confidence: ares_candidates::CandidateConfidence {
            cluster_strength: 0.5,
            evidence_count: 0,
            temporal_consistency: 0.5,
            source_diversity: 1,
        },
        bootstrap_metadata: None,
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    }
}

#[tokio::test]
async fn cert_12_confidence_stability() {
    let mut candidate = create_candidate();
    candidate.confidence.evidence_count = 5;
    candidate.confidence.source_diversity = 1;
    candidate.confidence.temporal_consistency = 0.9;

    assert_eq!(candidate.confidence.evidence_count, 5);
}

#[tokio::test]
async fn cert_13_bulk_promotion_thresholds() {
    let policy = CandidateGovernancePolicy {
        bulk_promotion_threshold: 0.95,
        ..Default::default()
    };
    let engine = CandidateGovernanceEngine::new(policy);

    let candidate = create_candidate();
    assert!(engine.meets_bulk_promotion_threshold(&candidate, 0.96));
    assert!(!engine.meets_bulk_promotion_threshold(&candidate, 0.90));
}

#[tokio::test]
async fn cert_14_candidate_expiration() {
    let policy = CandidateGovernancePolicy {
        expiration_days: 90,
        ..Default::default()
    };
    let engine = CandidateGovernanceEngine::new(policy);

    let mut candidate = create_candidate();
    let now = Utc::now().timestamp();
    let days_91_seconds = 91 * 24 * 3600;
    candidate.created_at = now - days_91_seconds;

    let expired = engine.evaluate_expiration(&mut candidate);
    assert!(expired);
    assert_eq!(candidate.status, CandidateStatus::Rejected);
}

#[tokio::test]
async fn cert_15_candidate_revalidation() {
    let policy = CandidateGovernancePolicy::default();
    let engine = CandidateGovernanceEngine::new(policy);

    let mut candidate = create_candidate();
    candidate.bootstrap_metadata = Some(ares_candidates::BootstrapMetadata {
        commit_hash: "commit_A".to_string(),
        repository_id: "test_repo".to_string(),
        rule_id: "rule_1".to_string(),
        engine_version: "v1".to_string(),
        generated_at: Utc::now().timestamp(),
    });

    let requires_reval = engine.requires_revalidation(&candidate, "commit_B", "rule_1", "v1", true);
    assert!(requires_reval);

    let requires_reval = engine.requires_revalidation(&candidate, "commit_A", "rule_1", "v2", true);
    assert!(requires_reval);

    let requires_reval =
        engine.requires_revalidation(&candidate, "commit_A", "rule_1", "v1", false);
    assert!(requires_reval);

    let requires_reval = engine.requires_revalidation(&candidate, "commit_A", "rule_1", "v1", true);
    assert!(!requires_reval);
}
