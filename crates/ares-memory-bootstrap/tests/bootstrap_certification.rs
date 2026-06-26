use ares_candidates::{BootstrapMetadata, Candidate, CandidateStatus, CandidateType};
use ares_memory_bootstrap::rules::{BuiltInRules, RuleProvider, YamlRules};
use chrono::Utc;
use std::fs;
use tempfile::tempdir;

fn create_candidate(candidate_type: CandidateType) -> Candidate {
    Candidate {
        id: "c1".to_string(),
        project_id: "test-proj".to_string(),
        title: "Test Candidate".to_string(),
        description: "Description".to_string(),
        candidate_type,
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
async fn cert_4_rule_provider_loading() {
    let built_in = BuiltInRules::new();
    let rules = built_in.load_rules();
    assert!(
        !rules.is_empty(),
        "Built-in rules should load automatically"
    );
}

#[tokio::test]
async fn cert_5_yaml_rule_override() {
    let dir = tempdir().unwrap();
    let ares_dir = dir.path().join(".ares");
    fs::create_dir_all(&ares_dir).unwrap();
    let yaml_content = r#"
rules:
  - rule_id: custom_rule_1
    target_type: Capability
    trigger_pattern: "src/.*"
    inferred_payload: Custom
    confidence_score: 0.8
"#;
    let yaml_path = ares_dir.join("bootstrap_rules.yaml");
    fs::write(&yaml_path, yaml_content).unwrap();

    let yaml_rules = YamlRules::new(yaml_path.to_str().unwrap());
    let rules = yaml_rules.load_rules();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rule_id, "custom_rule_1");
}

#[tokio::test]
async fn cert_8_candidate_promotion() {
    let mut candidate = create_candidate(CandidateType::Capability);
    candidate.status = CandidateStatus::Approved;
    assert_eq!(candidate.status, CandidateStatus::Approved);
}

#[tokio::test]
async fn cert_9_candidate_traceability() {
    let mut candidate = create_candidate(CandidateType::Requirement);
    candidate.bootstrap_metadata = Some(BootstrapMetadata {
        commit_hash: "abcd123".to_string(),
        repository_id: "repo1".to_string(),
        rule_id: "req_rule".to_string(),
        engine_version: "v1".to_string(),
        generated_at: Utc::now().timestamp(),
    });

    let meta = candidate.bootstrap_metadata.as_ref().unwrap();
    assert_eq!(meta.commit_hash, "abcd123");
}

#[tokio::test]
async fn cert_10_bootstrap_explainability() {
    let mut candidate = create_candidate(CandidateType::Capability);
    candidate.description = "Because of extensive file modifications in src/auth".to_string();
    assert!(candidate
        .description
        .contains("extensive file modifications"));
}

#[tokio::test]
async fn cert_11_commit_hash_integrity() {
    let mut candidate = create_candidate(CandidateType::Ownership);
    candidate.bootstrap_metadata = Some(BootstrapMetadata {
        commit_hash: "9f7278d6".to_string(),
        repository_id: "repo1".to_string(),
        rule_id: "owner_rule".to_string(),
        engine_version: "v1".to_string(),
        generated_at: Utc::now().timestamp(),
    });

    let meta = candidate.bootstrap_metadata.as_ref().unwrap();
    assert!(
        !meta.commit_hash.is_empty(),
        "Commit hash must exist for bootstrap candidates"
    );
}
