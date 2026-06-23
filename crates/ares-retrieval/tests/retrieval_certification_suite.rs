use ares_core::types::event::now_micros;
use ares_core::types::node::{EdgeType, GraphEdge, GraphNode, NodeType};
use ares_core::{NodeId, Project, ProjectId, ProjectMaturity};
use ares_retrieval::context_builder::ContextBuilder;
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;
use ares_retrieval::models::RankingWeights;
use ares_retrieval::query_engine::QueryEngine;
use ares_retrieval::retrieval_ranking_engine::RetrievalRankingEngine;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::project::SqliteProjectRepository;
use ares_store::Store;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

fn create_store() -> Arc<Store> {
    let db_path = format!("test_db_{}.sqlite", Uuid::new_v4());
    let store = Store::open(&PathBuf::from(&db_path)).expect("Failed to create store");
    Arc::new(store)
}

fn create_project(store: &Arc<Store>, id: &str) -> ProjectId {
    let repo = SqliteProjectRepository::new((**store).clone());
    let p_id = ProjectId::from(id);
    let now = now_micros();
    repo.create(&Project {
        id: p_id.clone(),
        name: "Test".to_string(),
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
fn cert_1_retrieval_determinism() {
    let store = create_store();
    let p_id = create_project(&store, "proj_1");

    create_node(
        &store,
        &p_id,
        NodeType::Requirement,
        "Req 1",
        serde_json::json!({}),
    );
    create_node(
        &store,
        &p_id,
        NodeType::Requirement,
        "Req 2",
        serde_json::json!({}),
    );

    let engine = MemoryRetrievalEngine::new(store.clone());

    let run1 = engine.find_by_type(&p_id, NodeType::Requirement).unwrap();
    let run2 = engine.find_by_type(&p_id, NodeType::Requirement).unwrap();
    let run3 = engine.find_by_type(&p_id, NodeType::Requirement).unwrap();

    assert_eq!(run1.len(), 2);
    // Ensure lists are identical in content and order
    for i in 0..2 {
        assert_eq!(run1[i].id, run2[i].id);
        assert_eq!(run2[i].id, run3[i].id);
    }
}

#[test]
fn cert_2_repository_isolation() {
    let store = create_store();
    let p1 = create_project(&store, "proj_a");
    let p2 = create_project(&store, "proj_b");

    create_node(
        &store,
        &p1,
        NodeType::Feature,
        "Feat A",
        serde_json::json!({ "has_drift": true }),
    );
    create_node(
        &store,
        &p2,
        NodeType::Feature,
        "Feat B",
        serde_json::json!({ "has_drift": true }),
    );

    let engine = MemoryRetrievalEngine::new(store.clone());
    let query = QueryEngine::new(&engine);

    let drift_p1 = query.find_drifted_capabilities(&p1).unwrap();
    let drift_p2 = query.find_drifted_capabilities(&p2).unwrap();

    assert_eq!(drift_p1.len(), 1);
    assert_eq!(drift_p1[0].label, "Feat A");

    assert_eq!(drift_p2.len(), 1);
    assert_eq!(drift_p2[0].label, "Feat B");
}

#[test]
fn cert_3_context_completeness() {
    let store = create_store();
    let p = create_project(&store, "proj_ctx");

    let req = create_node(
        &store,
        &p,
        NodeType::Requirement,
        "Req",
        serde_json::json!({ "owners": ["Team A"] }),
    );
    let dec = create_node(
        &store,
        &p,
        NodeType::Decision,
        "Dec",
        serde_json::json!({ "approvers": ["John", "Jane"] }),
    );

    create_edge(&store, &p, &req, &dec, EdgeType::Drives);

    let engine = MemoryRetrievalEngine::new(store.clone());
    let builder = ContextBuilder::new(&engine);

    let pack = builder.build_context_pack(&dec).unwrap();

    assert!(pack.requirement.is_some());
    assert_eq!(pack.requirement.unwrap().id.to_string(), req);
    assert_eq!(pack.decisions.len(), 1);
    assert_eq!(pack.decisions[0].id.to_string(), dec);

    // Check governance
    assert_eq!(pack.governance.approvers.len(), 2);
}

#[test]
fn cert_4_deep_graph_retrieval() {
    let store = create_store();
    let p = create_project(&store, "proj_deep");

    let mut nodes = vec![];
    for i in 0..10 {
        nodes.push(create_node(
            &store,
            &p,
            NodeType::Architecture,
            &format!("Node {}", i),
            serde_json::json!({}),
        ));
    }

    // Chain them: 0 -> 1 -> 2 ... -> 9
    for i in 0..9 {
        create_edge(&store, &p, &nodes[i], &nodes[i + 1], EdgeType::DependsOn);
    }

    let engine = MemoryRetrievalEngine::new(store.clone());

    let neighbors = engine
        .get_neighborhood(
            &nodes[0],
            ares_core::EdgeDirection::Outgoing,
            &[EdgeType::DependsOn],
        )
        .unwrap();

    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].id.to_string(), nodes[1]);
}

#[test]
fn cert_5_ranking_stability() {
    let store = create_store();
    let p = create_project(&store, "proj_rank");

    let _n1 = create_node(
        &store,
        &p,
        NodeType::Decision,
        "Dec 1",
        serde_json::json!({ "owners": ["Team A"], "approvers": ["John"] }),
    );
    let _n2 = create_node(
        &store,
        &p,
        NodeType::Decision,
        "Dec 2",
        serde_json::json!({ "owners": ["Team A"] }),
    );
    let _n3 = create_node(
        &store,
        &p,
        NodeType::Decision,
        "Dec 3",
        serde_json::json!({}),
    );

    let engine = MemoryRetrievalEngine::new(store.clone());
    let nodes = engine.find_by_type(&p, NodeType::Decision).unwrap();

    let weights = RankingWeights {
        authority: 1.0,
        traceability: 1.0,
        governance: 1.0,
        freshness: 0.5,
        completeness: 1.0,
    };
    let ranker = RetrievalRankingEngine::new(weights);

    let ranked = ranker.rank_nodes(&nodes);

    // Expected order: Dec 1 (has owners + approvers) > Dec 2 (has owners) > Dec 3 (none)
    assert_eq!(ranked.len(), 3);
    assert_eq!(ranked[0].node.label, "Dec 1");
    assert_eq!(ranked[1].node.label, "Dec 2");
    assert_eq!(ranked[2].node.label, "Dec 3");
}

#[test]
fn cert_6_large_graph_performance() {
    let store = create_store();
    let p = create_project(&store, "proj_perf");

    // We can't realistically test millions of nodes in a quick unit test without huge setup time,
    // but we can create 1000 nodes and ensure the retrieval is fast enough.
    let start = std::time::Instant::now();
    for i in 0..1000 {
        create_node(
            &store,
            &p,
            NodeType::File,
            &format!("File {}", i),
            serde_json::json!({}),
        );
    }

    let engine = MemoryRetrievalEngine::new(store.clone());
    let nodes = engine.find_by_type(&p, NodeType::File).unwrap();

    let elapsed = start.elapsed();

    assert_eq!(nodes.len(), 1000);
    assert!(
        elapsed.as_secs_f64() < 5.0,
        "Large graph creation and query took too long: {:?}",
        elapsed
    );
}
