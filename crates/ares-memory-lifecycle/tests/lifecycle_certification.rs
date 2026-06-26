use ares_memory_lifecycle::engines::{LifecycleEngine, LifecycleInput};
use ares_memory_lifecycle::models::{LifecycleState, SupersessionRecord};

fn setup_engine() -> LifecycleEngine {
    // defaults from configs/lifecycle.yaml
    LifecycleEngine::new(3, 6, 2, 180)
}

fn default_input() -> LifecycleInput {
    LifecycleInput {
        artifact_id: "test-artifact".to_string(),
        days_since_last_validation: 0,
        baseline_change_frequency_days: 30,
        evidence_count: 2,
        manual_approvals: 0,
        revalidation_successes: 0,
        contradiction_signals: 0,
        is_orphaned: false,
        is_unused: false,
        supersession_records: vec![],
        days_since_superseded: None,
        revalidation_attempted: false,
        revalidation_successful: false,
    }
}

#[test]
fn cert_1_freshness_score() {
    let engine = setup_engine();
    let mut input = default_input();
    input.days_since_last_validation = 10;
    input.baseline_change_frequency_days = 30; // stale at 90, decay at 180
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Fresh);
    assert_eq!(report.freshness.score, 1.0);
}

#[test]
fn cert_2_trust_score() {
    let engine = setup_engine();
    let mut input = default_input();
    input.evidence_count = 2; // >= minimum_evidence (2)
    let report = engine.evaluate(input);
    assert!(report.trust.is_trusted);
    assert_eq!(report.trust.score, 0.5);
}

#[test]
fn cert_3_decay_detection() {
    let engine = setup_engine();
    let mut input = default_input();
    input.is_orphaned = true; // immediately decaying
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Decaying);
}

#[test]
fn cert_4_archival() {
    let engine = setup_engine();
    let mut input = default_input();
    input.supersession_records = vec![SupersessionRecord::new(
        "old".into(),
        "new".into(),
        "reason".into(),
        0.9,
    )];
    input.days_since_superseded = Some(181); // threshold is 180
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Archived);
    assert!(report.is_archivable);
}

#[test]
fn cert_5_supersession() {
    let engine = setup_engine();
    let mut input = default_input();
    input.supersession_records = vec![SupersessionRecord::new(
        "old".into(),
        "new".into(),
        "reason".into(),
        0.9,
    )];
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Superseded);
}

#[test]
fn cert_6_determinism() {
    let engine = setup_engine();
    let input1 = default_input();
    let input2 = default_input();
    let r1 = engine.evaluate(input1);
    let r2 = engine.evaluate(input2);
    assert_eq!(r1, r2);
}

#[test]
fn cert_7_repository_isolation() {
    let engine = setup_engine();
    let mut input = default_input();
    input.artifact_id = "repoA::artifact".to_string();
    let report = engine.evaluate(input);
    assert_eq!(report.artifact_id, "repoA::artifact");
}

#[test]
fn cert_8_explainability() {
    let engine = setup_engine();
    let mut input = default_input();
    input.contradiction_signals = 1;
    let report = engine.evaluate(input);
    assert_eq!(report.trust.contradiction_signals, 1);
    assert!(!report.trust.is_trusted); // contradiction removes trust
}

#[test]
fn cert_9_memory_health() {
    let engine = setup_engine();
    let mut input = default_input();
    input.days_since_last_validation = 91; // stale
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Stale);
}

#[test]
fn cert_10_lifecycle_transition() {
    let engine = setup_engine();
    let mut input = default_input();
    input.days_since_last_validation = 181; // decaying
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Decaying);
}

#[test]
fn cert_11_candidate_freshness() {
    let engine = setup_engine();
    let mut input = default_input();
    input.days_since_last_validation = 0;
    let report = engine.evaluate(input);
    assert!(!report.freshness.is_stale);
}

#[test]
fn cert_12_candidate_decay() {
    let engine = setup_engine();
    let mut input = default_input();
    input.days_since_last_validation = 200; // decaying
    let report = engine.evaluate(input);
    assert!(report.freshness.is_decaying);
}

#[test]
fn cert_13_archived_queryability() {
    let engine = setup_engine();
    let mut input = default_input();
    input.is_unused = true;
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Archived);
    assert!(report.is_archivable);
}

#[test]
fn cert_14_trust_recalculation() {
    let engine = setup_engine();
    let mut input = default_input();
    input.manual_approvals = 1;
    let report = engine.evaluate(input);
    assert_eq!(report.trust.score, 0.8); // 0.5 (evidence) + 0.3 (approval)
}

#[test]
fn cert_15_memory_governance() {
    // Tests that engine correctly loads config bounds
    let engine = setup_engine();
    assert_eq!(engine.trust_engine.minimum_evidence, 2);
    assert_eq!(engine.archival_engine.superseded_after_days, 180);
}

#[test]
fn cert_16_revalidation_success() {
    let engine = setup_engine();
    let mut input = default_input();
    input.days_since_last_validation = 91; // stale
    input.revalidation_attempted = true;
    input.revalidation_successful = true;
    let report = engine.evaluate(input);
    // Successful revalidation of stale memory returns it to fresh
    assert_eq!(report.current_state, LifecycleState::Fresh);
}

#[test]
fn cert_17_revalidation_failure() {
    let engine = setup_engine();
    let mut input = default_input();
    input.days_since_last_validation = 91; // stale
    input.revalidation_attempted = true;
    input.revalidation_successful = false;
    let report = engine.evaluate(input);
    // Failed revalidation makes it decaying. (Decaying only becomes archived if unused).
    assert_eq!(report.current_state, LifecycleState::Decaying);
}

#[test]
fn cert_18_adaptive_freshness() {
    let engine = setup_engine();
    let mut input = default_input();
    input.baseline_change_frequency_days = 5; // Fast changing repo (e.g. Tokio)
    input.days_since_last_validation = 16; // 16 > 3 * 5
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Stale);
}

#[test]
fn cert_19_generic_supersession() {
    let engine = setup_engine();
    let mut input = default_input();
    input.supersession_records = vec![
        SupersessionRecord::new(
            "cap1".into(),
            "cap2".into(),
            "deprecated feature".into(),
            0.95,
        ),
        SupersessionRecord::new(
            "req1".into(),
            "req2".into(),
            "updated requirements".into(),
            0.80,
        ),
    ];
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Superseded);
    assert_eq!(report.supersession.unwrap().old_node, "cap1"); // Picked highest confidence
}

#[test]
fn cert_20_lifecycle_recovery() {
    let engine = setup_engine();
    let mut input = default_input();
    input.is_orphaned = true; // Becomes Decaying
    let report = engine.evaluate(input);
    assert_eq!(report.current_state, LifecycleState::Decaying);

    // Recovery via fixing unused/orphaned
    let mut input2 = default_input();
    input2.is_orphaned = false;
    let report2 = engine.evaluate(input2);
    assert_eq!(report2.current_state, LifecycleState::Fresh);
}
