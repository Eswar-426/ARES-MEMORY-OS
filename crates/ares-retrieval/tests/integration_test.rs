use ares_retrieval::memory::{
    engine::RetrievalEngine,
    planner::MemoryQueryPlanner,
    source::{
        DecisionMemorySource, GapMemorySource, MemorySourceRegistry, RequirementMemorySource,
        ResolutionMemorySource,
    },
};
use ares_store::Store;
use std::sync::Arc;
use tempfile::tempdir;

fn setup_store() -> Store {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_retrieval.db");
    Store::open(&db_path).unwrap()
}

fn seed_data(store: &Store) {
    let conn = store.get_conn().unwrap();

    // Seed Requirement
    conn.execute(
        "INSERT INTO requirements (
            id, project_id, title, description, requirement_type, status, priority, source, created_at, updated_at
        ) VALUES (
            'REQ-AUTH', 'PROJ-DEFAULT', 'Authentication System', 'Why does authentication exist? We need user login.', 
            'security', 'approved', 'high', 'Stakeholder', 1622500000000, 1622500000000
        )",
        [],
    ).unwrap();

    // Seed Decision
    conn.execute(
        "INSERT INTO decision_records (
            id, title, context, problem, chosen_option,
            rejected_options, assumptions, consequences, confidence, owner,
            approval_status, approved_by, approved_at, created_at, updated_at
        ) VALUES (
            'DEC-AUTH-SYS', 'Approve Authentication Architecture', 'Why was authentication approved? Fast and stateless.', 'Need secure login', 'JWT',
            '[]', '[]', '[]', '\"high\"', 'Bob',
            '\"approved\"', 'Bob', 1622500000000, 1622500000000, 1622500000000
        )",
        [],
    ).unwrap();

    // Evidence
    conn.execute(
        "INSERT INTO decision_evidence (
            id, decision_id, source, reference_url, description, confidence_score
        ) VALUES (
            'EVID-1', 'DEC-AUTH-SYS', '\"Benchmark\"', '', 'JWT vs Session benchmark', 0.9
        )",
        [],
    )
    .unwrap();

    // Outcome
    conn.execute(
        "INSERT INTO decision_outcomes (
            id, decision_id, observed_at, description, outcome_type, success_score
        ) VALUES (
            'OUT-1', 'DEC-AUTH-SYS', 1622500000000, 'Authentication login works.', '\"Success\"', 1.0
        )",
        [],
    ).unwrap();
}

#[test]
fn test_end_to_end_retrieval() {
    let store = setup_store();
    seed_data(&store);

    let mut registry = MemorySourceRegistry::new(store.clone());
    registry.register(Arc::new(RequirementMemorySource));
    registry.register(Arc::new(DecisionMemorySource));
    registry.register(Arc::new(GapMemorySource));
    registry.register(Arc::new(ResolutionMemorySource));

    let planner = MemoryQueryPlanner::default();
    let engine = RetrievalEngine::new(planner, registry);

    // Test 1: Requirement Retrieval
    let res1 = engine.retrieve("Why does authentication exist?");
    assert_eq!(res1.context.requirements.len(), 1, "Expected 1 requirement");
    assert_eq!(res1.context.requirements[0].id.as_str(), "REQ-AUTH");

    // Test 2: Decision Retrieval
    let res2 = engine.retrieve("Why was authentication approved?");
    assert_eq!(res2.context.decisions.len(), 1, "Expected 1 decision");
    assert_eq!(res2.context.decisions[0].id.as_str(), "DEC-AUTH-SYS");
    assert_eq!(res2.context.evidence.len(), 1, "Expected 1 evidence");
    assert_eq!(res2.context.outcomes.len(), 1, "Expected 1 outcome");

    // Test 3: Gap Retrieval
    let res3 = engine.retrieve("What knowledge debt exists?");
    assert!(!res3.context.gaps.is_empty(), "Expected gaps to be found");
    assert!(res3.context.knowledge_debt.is_some());

    // Test 4: Resolution Retrieval
    let res4 = engine.retrieve("How should we fix resolution gap debt?");
    assert!(
        !res4.context.resolution_plans.is_empty(),
        "Expected resolution plans"
    );

    // Test 5: Health Retrieval
    let res5 = engine.retrieve("Repository health score");
    assert!(res5.context.health_report.is_some());

    // Context Quality Validation
    assert!(res2.context.context_quality.is_some());
    let quality = res2.context.context_quality.unwrap();
    assert!(
        quality.traceability_score > 0.0,
        "Expected traceability score > 0"
    );
    assert!(
        quality.completeness_score > 0.0,
        "Expected completeness score > 0"
    );
    assert!(
        res2.explanation.coverage.decisions_found > 0,
        "Expected coverage > 0"
    );
}
