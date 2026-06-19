use ares_core::id::{new_id, ProjectId, EvidenceId, RequirementLinkId};
use ares_core::{DecisionId, RequirementId};
use ares_decision_intelligence::health::DecisionHealthEngine;
use ares_decision_intelligence::history::DecisionHistory;
use ares_decision_intelligence::models::{
    Decision, DecisionConfidence, DecisionEvidence, DecisionOutcome, DecisionStatus, EvidenceSource,
    OutcomeType,
};
use ares_decision_intelligence::storage::DecisionStore;
use ares_requirements::health::RequirementHealthEngine;
use ares_requirements::models::{
    CreateRequirementInput, LinkTarget, RequirementLink, RequirementPriority, RequirementType, RequirementSource,
};
use ares_requirements::storage::RequirementStore;
use ares_store::Store;

fn setup_store() -> Store {
    let db_path = std::env::temp_dir().join(format!("ares_test_{}.db", new_id()));
    Store::open(&db_path).expect("Failed to open DB")
}

#[tokio::test]
async fn test_gap_engine_validation_suite() {
    let store = setup_store();
    let project_id = ProjectId::from(new_id());

    // Initialize stores
    let req_store = RequirementStore::new(store.clone());
    let dec_store = DecisionStore::new(store.clone());
    let dec_history = DecisionHistory::new(store.clone());
    let req_health = RequirementHealthEngine::new(store.clone());
    let dec_health = DecisionHealthEngine::new(store.clone());

    // ─────────────────────────────────────────────────────────────────
    // 1. Requirement ➔ Decision Traceability
    // ─────────────────────────────────────────────────────────────────

    // Create a Requirement
    let input = CreateRequirementInput {
        project_id: project_id.clone(),
        title: "Support RBAC".to_string(),
        description: "Need Role-Based Access Control".to_string(),
        source: RequirementSource::Security,
        requirement_type: RequirementType::Security,
        priority: RequirementPriority::High,
        owner: Some("Alice".to_string()),
        tags: vec![],
    };
    let req = req_store.create(input).unwrap();

    // Create a Decision
    let mut dec = Decision {
        id: DecisionId::from(new_id()),
        title: "Use Casbin for RBAC".to_string(),
        context: "We need RBAC".to_string(),
        problem: "How to implement it?".to_string(),
        chosen_option: "Casbin".to_string(),
        rejected_options: vec![],
        assumptions: vec![],
        consequences: vec![],
        confidence: DecisionConfidence::High,
        owner: Some("Alice".to_string()),
        approval_status: DecisionStatus::Proposed,
        approved_by: None,
        approved_at: None,
        created_at: 1000,
        updated_at: 1000,
    };
    dec_store.insert_decision(&dec).unwrap();

    // Link Requirement to Decision
    let link = RequirementLink {
        id: RequirementLinkId::from(new_id()),
        source_requirement_id: req.id.clone(),
        target: LinkTarget::Decision(dec.id.clone()),
        relationship: "implements".to_string(),
        created_at: 1000,
        created_by: Some("Alice".to_string()),
    };
    req_store.create_link(&link).unwrap();

    // Add Evidence
    let ev = DecisionEvidence {
        id: EvidenceId::from(new_id()),
        decision_id: dec.id.clone(),
        source: EvidenceSource::RFC,
        reference_url: "https://example.com/rfc1".to_string(),
        description: "RFC approval".to_string(),
        confidence_score: 1.0,
    };
    dec_store.insert_evidence(&ev).unwrap();

    // Validate Traceability
    let filter = ares_requirements::models::RequirementFilter {
        status: None,
        priority: None,
        requirement_type: None,
        owner: None,
        tag: None,
        since: None,
        until: None,
    };
    let linked_reqs = req_store.list(&project_id, filter).unwrap();
    assert_eq!(linked_reqs.len(), 1);
    
    let links = req_store.get_links_from(&req.id).unwrap();
    assert_eq!(links.len(), 1);

    let dec_ev = dec_store.get_evidence(&dec.id).unwrap();
    assert_eq!(dec_ev.len(), 1);

    // ─────────────────────────────────────────────────────────────────
    // 2. Decision Timeline Reconstruction
    // ─────────────────────────────────────────────────────────────────

    dec.approval_status = DecisionStatus::Approved;
    dec.approved_by = Some("Bob".to_string());
    dec.updated_at = 2000;
    
    // Record revision
    let diff = "Status changed to Approved".to_string();
    dec_history.record_revision(&dec.id, Some("Bob"), Some("Approval granted"), &diff).unwrap();

    // Add Outcome
    let outcome = DecisionOutcome {
        id: new_id(),
        decision_id: dec.id.clone(),
        observed_at: 3000,
        description: "System successfully uses Casbin now".to_string(),
        outcome_type: OutcomeType::Success,
        success_score: Some(0.9),
    };
    dec_store.insert_outcome(&outcome).unwrap();

    // Fetch History
    let revisions = dec_history.get_revisions(&dec.id).unwrap();
    assert_eq!(revisions.len(), 1);

    let outcomes = dec_store.get_outcomes(&dec.id).unwrap();
    assert_eq!(outcomes.len(), 1);

    // ─────────────────────────────────────────────────────────────────
    // 3. Repository-Wide Health Scoring
    // ─────────────────────────────────────────────────────────────────

    // Create a Stale/Orphan Requirement
    let orphan_req_input = CreateRequirementInput {
        project_id: project_id.clone(),
        title: "Unimplemented stuff".to_string(),
        description: "Nobody uses this".to_string(),
        source: RequirementSource::Architecture,
        requirement_type: RequirementType::Functional,
        priority: RequirementPriority::Low,
        owner: Some("Alice".to_string()),
        tags: vec![],
    };
    let orphan_req = req_store.create(orphan_req_input).unwrap();

    let req_health_snapshot = req_health.compute_health(&project_id).unwrap();
    // We expect a score based on 2 requirements, 1 without decision
    assert!(req_health_snapshot.decision_coverage_score < 100.0);
    // At least 1 issue should be NoDecision
    let has_no_decision_issue = req_health_snapshot.issues.iter().any(|i| {
        i.requirement_id == orphan_req.id && i.issue_type == ares_requirements::health::HealthIssueType::NoDecision
    });
    assert!(has_no_decision_issue);
    
    let dec_health_snapshot = dec_health.generate_snapshot(&project_id).unwrap();
    assert_eq!(dec_health_snapshot.total_decisions, 1);
    assert_eq!(dec_health_snapshot.decisions_with_evidence, 1);

    // If we add a decision without evidence, the health should drop
    let bad_dec = Decision {
        id: DecisionId::from(new_id()),
        title: "Bad Decision".to_string(),
        context: "".to_string(),
        problem: "".to_string(),
        chosen_option: "".to_string(),
        rejected_options: vec![],
        assumptions: vec![],
        consequences: vec![],
        confidence: DecisionConfidence::Experimental,
        owner: None, // Missing owner!
        approval_status: DecisionStatus::Proposed,
        approved_by: None,
        approved_at: None,
        created_at: 1000,
        updated_at: 1000,
    };
    dec_store.insert_decision(&bad_dec).unwrap();

    let dec_health_snapshot_2 = dec_health.generate_snapshot(&project_id).unwrap();
    assert_eq!(dec_health_snapshot_2.total_decisions, 2);
    assert_eq!(dec_health_snapshot_2.decisions_without_owner, 1);
    assert_eq!(dec_health_snapshot_2.decisions_with_evidence, 1); // Only 1 has evidence out of 2

    println!("All validation checks passed!");
}
