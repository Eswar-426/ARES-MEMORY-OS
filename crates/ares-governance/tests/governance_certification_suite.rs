use ares_core::types::event::now_micros;
use ares_core::types::node::{EdgeType, GraphEdge, GraphNode, NodeType};
use ares_core::{NodeId, Project, ProjectId, ProjectMaturity};
use ares_governance::governance_gap_engine::{
    GovernanceGapEngine, GovernanceGapType, GovernanceSeverity,
};
use ares_governance::governance_health_engine::GovernanceHealthEngine;
use ares_governance::ownership_engine::OwnershipEngine;
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
) -> String {
    let repo = SqliteGraphRepository::new((**store).clone());
    let id = Uuid::new_v4().to_string();
    let now = now_micros();
    repo.upsert_node(GraphNode {
        id: NodeId::from(id.clone()),
        project_id: project_id.clone(),
        node_type,
        label: label.to_string(),
        properties: serde_json::json!({}),
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
fn cert_1_ownership_inheritance() {
    let store = create_store();
    let p_id = create_project(&store, "proj_1");

    let team_a = create_node(&store, &p_id, NodeType::Team, "Team Alpha");
    let req_1 = create_node(&store, &p_id, NodeType::Requirement, "Req 1");
    let dec_1 = create_node(&store, &p_id, NodeType::Decision, "Dec 1");

    // Team owns Req, Req drives Dec
    create_edge(&store, &p_id, &team_a, &req_1, EdgeType::Owns);
    create_edge(&store, &p_id, &req_1, &dec_1, EdgeType::Drives);

    let ownership = OwnershipEngine::new(store.clone());

    let req_owner = ownership.resolve_owner(&req_1).unwrap().unwrap();
    assert_eq!(req_owner.owner_id, team_a);
    assert!(req_owner.is_explicit);

    let dec_owner = ownership.resolve_owner(&dec_1).unwrap().unwrap();
    assert_eq!(dec_owner.owner_id, team_a);
    assert!(!dec_owner.is_explicit); // Inherited
}

#[test]
fn cert_2_ownership_override() {
    let store = create_store();
    let p_id = create_project(&store, "proj_2");

    let team_a = create_node(&store, &p_id, NodeType::Team, "Team Alpha");
    let team_b = create_node(&store, &p_id, NodeType::Team, "Team Beta");
    let person_c = create_node(&store, &p_id, NodeType::Person, "Person C");

    let req_1 = create_node(&store, &p_id, NodeType::Requirement, "Req 1");
    let dec_1 = create_node(&store, &p_id, NodeType::Decision, "Dec 1");

    // Inherited from Team A
    create_edge(&store, &p_id, &team_a, &req_1, EdgeType::Owns);
    create_edge(&store, &p_id, &req_1, &dec_1, EdgeType::Drives);

    // Explicit Team B
    create_edge(&store, &p_id, &team_b, &dec_1, EdgeType::Owns);

    let ownership = OwnershipEngine::new(store.clone());

    let dec_owner = ownership.resolve_owner(&dec_1).unwrap().unwrap();
    // Team B explicit override
    assert_eq!(dec_owner.owner_id, team_b);
    assert!(dec_owner.is_explicit);

    // If person C also owns it, Team B is authoritative
    create_edge(&store, &p_id, &person_c, &dec_1, EdgeType::Owns);
    let dec_owner2 = ownership.resolve_owner(&dec_1).unwrap().unwrap();
    assert_eq!(dec_owner2.owner_id, team_b); // Still Team B
}

#[test]
fn cert_3_orphan_governance_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_3");

    // No owners mapped
    let req_1 = create_node(&store, &p_id, NodeType::Requirement, "Orphan Req");

    let gap_engine = GovernanceGapEngine::new(store.clone());
    let gaps = gap_engine.analyze_node(&req_1).unwrap();

    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].gap_type, GovernanceGapType::MissingOwner);
    assert_eq!(gaps[0].severity, GovernanceSeverity::Critical);
}

#[test]
fn cert_4_governance_gap_detection() {
    let store = create_store();
    let p_id = create_project(&store, "proj_4");

    let dec_1 = create_node(&store, &p_id, NodeType::Decision, "Dec 1");
    let feat_1 = create_node(&store, &p_id, NodeType::Feature, "Feat 1");
    let arch_1 = create_node(&store, &p_id, NodeType::Architecture, "Arch 1");

    let gap_engine = GovernanceGapEngine::new(store.clone());

    let dec_gaps = gap_engine.analyze_node(&dec_1).unwrap();
    assert!(dec_gaps.iter().any(
        |g| g.gap_type == GovernanceGapType::MissingDecisionAuthority
            && g.severity == GovernanceSeverity::High
    ));

    let feat_gaps = gap_engine.analyze_node(&feat_1).unwrap();
    assert!(feat_gaps
        .iter()
        .any(|g| g.gap_type == GovernanceGapType::MissingCapabilityOwner
            && g.severity == GovernanceSeverity::Low));

    let arch_gaps = gap_engine.analyze_node(&arch_1).unwrap();
    assert!(arch_gaps
        .iter()
        .any(|g| g.gap_type == GovernanceGapType::MissingOwner
            && g.severity == GovernanceSeverity::Medium));
}

#[test]
fn cert_5_determinism() {
    let store = create_store();
    let p_id = create_project(&store, "proj_5");

    let team_a = create_node(&store, &p_id, NodeType::Team, "Team Alpha");
    let req_1 = create_node(&store, &p_id, NodeType::Requirement, "Req 1");
    create_edge(&store, &p_id, &team_a, &req_1, EdgeType::Owns);

    let ownership = OwnershipEngine::new(store.clone());

    let run_1 = ownership.resolve_owner(&req_1).unwrap();
    let run_2 = ownership.resolve_owner(&req_1).unwrap();
    let run_3 = ownership.resolve_owner(&req_1).unwrap();

    assert_eq!(run_1, run_2);
    assert_eq!(run_2, run_3);
}

#[test]
fn cert_6_repository_isolation() {
    let store = create_store();
    let p_1 = create_project(&store, "proj_6_1");
    let p_2 = create_project(&store, "proj_6_2");

    let _team_p1 = create_node(&store, &p_1, NodeType::Team, "Team 1");
    let req_p2 = create_node(&store, &p_2, NodeType::Requirement, "Req 2");

    let gap_engine = GovernanceGapEngine::new(store.clone());
    let req_p2_gaps = gap_engine.analyze_node(&req_p2).unwrap();
    assert_eq!(req_p2_gaps[0].gap_type, GovernanceGapType::MissingOwner);
}

#[test]
fn cert_7_governance_health_score() {
    let score1 = GovernanceHealthEngine::calculate(100.0, 100.0, 100.0);
    assert_eq!(score1.total_health, 100.0);

    let score2 = GovernanceHealthEngine::calculate(50.0, 50.0, 50.0);
    assert_eq!(score2.total_health, 50.0);

    let score3 = GovernanceHealthEngine::calculate(100.0, 0.0, 0.0);
    assert_eq!(score3.total_health, 40.0); // 40% weight
}
