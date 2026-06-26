use ares_core::types::event::now_micros;
use ares_core::types::node::{EdgeType, GraphEdge, GraphNode, NodeType};
use ares_core::{NodeId, Project, ProjectId, ProjectMaturity};
use ares_knowledge_gap::engine::KnowledgeGapEngine;
use ares_knowledge_gap::models::KnowledgeGapType;
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::project::SqliteProjectRepository;
use ares_store::Store;
use std::sync::Arc;
use uuid::Uuid;

fn create_store() -> Arc<Store> {
    let db_path = std::env::temp_dir().join(format!("test_db_{}.sqlite", Uuid::new_v4()));
    let store = Store::open(&db_path).expect("Failed to create store");
    Arc::new(store)
}

fn create_project(store: &Arc<Store>, id: &str) -> ProjectId {
    let repo = SqliteProjectRepository::new((**store).clone());
    let p_id = ProjectId::from(id);
    let now = now_micros();
    repo.create(&Project {
        id: p_id.clone(),
        name: "Test Project".to_string(),
        description: "Desc".to_string(),
        root_path: format!("/{}", id),
        primary_language: "rust".to_string(),
        domain: "test".to_string(),
        maturity: ProjectMaturity::Greenfield,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    })
    .unwrap();
    p_id
}

fn create_node(
    store: &Arc<Store>,
    project_id: &ProjectId,
    node_type: NodeType,
    label: &str,
    props: serde_json::Value,
) -> String {
    let repo = SqliteGraphRepository::new((**store).clone());
    let id = Uuid::new_v4().to_string();
    let now = now_micros();
    repo.upsert_node(GraphNode {
        id: NodeId::from(id.clone()),
        project_id: project_id.clone(),
        node_type,
        label: label.to_string(),
        properties: props,
        file_path: None,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    })
    .unwrap();
    id
}

fn create_edge(
    store: &Arc<Store>,
    project_id: &ProjectId,
    from: &str,
    to: &str,
    edge_type: EdgeType,
) {
    let repo = SqliteGraphRepository::new((**store).clone());
    let id = Uuid::new_v4().to_string();
    let now = now_micros();
    repo.upsert_edge(GraphEdge {
        id,
        project_id: project_id.clone(),
        from_node_id: NodeId::from(from.to_string()),
        to_node_id: NodeId::from(to.to_string()),
        edge_type,
        weight: 1.0,
        confidence: 1.0,
        source: "agent".to_string(),
        valid_from: now,
        valid_until: None,
        created_at: now,
    })
    .unwrap();
}

#[test]
fn cert_1_missing_requirement_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_1");
    // Decision with no upstream requirement
    create_node(
        &store,
        &p_id,
        NodeType::Decision,
        "Dec 1",
        serde_json::json!({}),
    );

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::MissingRequirement));
}

#[test]
fn cert_2_missing_decision_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_2");
    // Architecture with no upstream decision
    create_node(
        &store,
        &p_id,
        NodeType::Architecture,
        "Arch 1",
        serde_json::json!({}),
    );

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::MissingDecision));
}

#[test]
fn cert_3_missing_architecture_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_3");
    let dec = create_node(
        &store,
        &p_id,
        NodeType::Decision,
        "Dec 1",
        serde_json::json!({}),
    );
    let code = create_node(
        &store,
        &p_id,
        NodeType::File,
        "Code 1",
        serde_json::json!({}),
    );
    create_edge(&store, &p_id, &dec, &code, EdgeType::Drives);

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::MissingArchitecture));
}

#[test]
fn cert_4_missing_ownership_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_4");
    create_node(
        &store,
        &p_id,
        NodeType::Feature,
        "Feat 1",
        serde_json::json!({}),
    );

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::MissingOwnership));
}

#[test]
fn cert_5_missing_tests_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_5");
    create_node(
        &store,
        &p_id,
        NodeType::File,
        "Code 1",
        serde_json::json!({}),
    );

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::MissingTests));
}

#[test]
fn cert_6_missing_runtime_validation() {
    let store = create_store();
    let p_id = create_project(&store, "proj_6");

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(!gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::MissingRuntimeValidation));
}

#[test]
fn cert_7_missing_outcome_tracking() {
    let store = create_store();
    let p_id = create_project(&store, "proj_7");

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(!gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::MissingOutcomeTracking));
}

#[test]
fn cert_8_missing_traceability_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_8");
    create_node(
        &store,
        &p_id,
        NodeType::File,
        "Code isolated",
        serde_json::json!({}),
    );

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::MissingTraceability));
}

#[test]
fn cert_9_knowledge_blind_spot_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_9");
    let feat = create_node(
        &store,
        &p_id,
        NodeType::Feature,
        "Feat 1",
        serde_json::json!({"has_drift": true}),
    );

    for i in 0..51 {
        let other = create_node(
            &store,
            &p_id,
            NodeType::File,
            &format!("File {}", i),
            serde_json::json!({}),
        );
        create_edge(&store, &p_id, &feat, &other, EdgeType::Implements);
    }

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::KnowledgeBlindSpot));
}

#[test]
fn cert_10_knowledge_debt_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_10");

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    assert!(gaps
        .iter()
        .any(|g| g.gap_type == KnowledgeGapType::KnowledgeDebt));
}

#[test]
fn cert_11_gap_explainability() {
    let store = create_store();
    let p_id = create_project(&store, "proj_11");
    create_node(
        &store,
        &p_id,
        NodeType::Feature,
        "Feat 1",
        serde_json::json!({}),
    );

    let ret_engine = MemoryRetrievalEngine::new(store.clone());
    let engine = KnowledgeGapEngine::new(&ret_engine);
    let gaps = engine.scan_and_analyze(&p_id).unwrap();

    let ownership_gap = gaps
        .iter()
        .find(|g| g.gap_type == KnowledgeGapType::MissingOwnership)
        .unwrap();

    assert!(!ownership_gap.evidence.rationale.is_empty());
    assert!(!ownership_gap.remediation.recommended_action.is_empty());
    assert!(!ownership_gap.evidence.source_nodes.is_empty());
}
