use ares_completeness::completeness_engine::CompletenessEngine;
use ares_completeness::coverage_engine::CoverageEngine;
use ares_completeness::coverage_snapshot::CoverageSnapshotRepository;
use ares_completeness::health_engine::RepositoryHealthEngine;
use ares_completeness::models::CoverageSnapshot;
use ares_completeness::prioritization_engine::GapPrioritizationEngine;
use ares_completeness::topology_engine::TopologyEngine;
use ares_store::Store;
use tempfile::tempdir;

fn test_store() -> (Store, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let store = Store::open(&db_path).unwrap();
    store.run_migrations().unwrap();
    (store, dir)
}

fn setup_test_data(store: &Store) {
    let conn = store.get_conn().unwrap();
    // Insert projects
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES ('proj_1', 'P1', 'mature', '/p1', 'rust', 'mature', 'mature', 0, 0)",
        [],
    )
    .unwrap();

    // Insert Nodes
    // Full chain: R1 -> D1 -> A1 -> C1 -> T1 -> RS1 -> O1
    let nodes = vec![
        ("R1", "requirement"),
        ("D1", "decision"),
        ("A1", "architecture"),
        ("C1", "file"),
        ("T1", "test"),
        ("RS1", "runtime_signal"),
        ("O1", "outcome"),
        // Partial chain: R2 -> D2 -> A2
        ("R2", "requirement"),
        ("D2", "decision"),
        ("A2", "architecture"),
        // Orphaned: R3
        ("R3", "requirement"),
    ];

    for (id, nt) in nodes {
        conn.execute(
            "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES (?1, 'proj_1', ?2, ?1, '{}', 0, 0)",
            rusqlite::params![id, nt],
        )
        .unwrap();
    }

    // Insert Edges
    let edges = vec![
        ("R1", "D1"),
        ("D1", "A1"),
        ("A1", "C1"),
        ("C1", "T1"),
        ("T1", "RS1"),
        ("RS1", "O1"),
        ("R2", "D2"),
        ("D2", "A2"),
    ];

    for (edge_id, (from, to)) in (1..).zip(edges) {
        conn.execute(
            "INSERT INTO graph_edges (id, project_id, from_node_id, to_node_id, edge_type, valid_from, created_at) VALUES (?1, 'proj_1', ?2, ?3, 'evolves', 0, 0)",
            rusqlite::params![format!("e{}", edge_id), from, to],
        )
        .unwrap();
    }
}

#[tokio::test]
async fn cert_01_topology_engine_classification() {
    let (store, _dir) = test_store();
    setup_test_data(&store);
    let topology_engine = TopologyEngine::new(store.clone());

    let segments = topology_engine.evaluate_topology(&"proj_1".into()).unwrap();
    assert_eq!(segments.len(), 11);

    let r1 = segments.iter().find(|s| s.node_id == "R1").unwrap();
    assert_eq!(r1.state, ares_completeness::models::TopologyState::Complete);
    assert!(r1.missing_downstream.is_empty());

    let r2 = segments.iter().find(|s| s.node_id == "R2").unwrap();
    assert_eq!(r2.state, ares_completeness::models::TopologyState::Partial);
    assert_eq!(
        r2.missing_downstream,
        vec!["Code", "Test", "RuntimeSignal", "Outcome"]
    );

    let r3 = segments.iter().find(|s| s.node_id == "R3").unwrap();
    assert_eq!(r3.state, ares_completeness::models::TopologyState::Orphaned);
}

#[tokio::test]
async fn cert_02_coverage_engine() {
    let (store, _dir) = test_store();
    setup_test_data(&store);
    let topology_engine = TopologyEngine::new(store.clone());
    let segments = topology_engine.evaluate_topology(&"proj_1".into()).unwrap();

    let coverage_engine = CoverageEngine::new();
    let metrics = coverage_engine.calculate_coverage(&segments);

    // 3 requirements (R1, R2, R3). R1 and R2 have decisions. 2/3 = 66.6%
    assert!((metrics.requirement_coverage - 66.66).abs() < 0.1);
    // 2 decisions (D1, D2). Both have architecture. 2/2 = 100%
    assert!((metrics.decision_coverage - 100.0).abs() < 0.1);
    // 2 architectures (A1, A2). A1 has code, A2 does not. 1/2 = 50%
    assert!((metrics.architecture_coverage - 50.0).abs() < 0.1);
    // 1 code, 1 test, 1 run => all 100%
    assert_eq!(metrics.code_coverage, 100.0);
    assert_eq!(metrics.test_coverage, 100.0);
    assert_eq!(metrics.runtime_coverage, 100.0);
}

#[tokio::test]
async fn cert_03_completeness_engine() {
    let (store, _dir) = test_store();
    setup_test_data(&store);
    let topology_engine = TopologyEngine::new(store.clone());
    let segments = topology_engine.evaluate_topology(&"proj_1".into()).unwrap();

    let completeness_engine = CompletenessEngine::new();
    let gaps = completeness_engine.find_completeness_gaps(&segments);

    // Should find gaps for R2, D2, A2 (partial) and R3 (orphaned).
    assert!(!gaps.is_empty());
    let score = completeness_engine.calculate_completeness_score(&segments);
    // R1, D1, A1, C1, T1, RS1, O1 are Complete (7). 7 / 11 = ~63%
    assert!((score - 63.63).abs() < 0.1);
}

#[tokio::test]
async fn cert_04_prioritization_engine() {
    let (store, _dir) = test_store();
    setup_test_data(&store);
    let topology_engine = TopologyEngine::new(store.clone());
    let segments = topology_engine.evaluate_topology(&"proj_1".into()).unwrap();

    let completeness_engine = CompletenessEngine::new();
    let gaps = completeness_engine.find_completeness_gaps(&segments);

    let prio_engine = GapPrioritizationEngine::new();
    let ranked = prio_engine.prioritize_gaps(gaps).await;

    // Highest risk should be Critical or High, lowest risk should be at the end.
    assert!(!ranked.is_empty());
    assert!(ranked[0].total_risk_score >= ranked.last().unwrap().total_risk_score);
}

#[tokio::test]
async fn cert_05_health_engine_weighted() {
    let health_engine = RepositoryHealthEngine::new();
    let score = health_engine.calculate_health("proj_1", 100.0, 100.0, 100.0, 100.0);
    assert_eq!(score.total_health, 100.0);

    // Weights: Cov=35, Comp=35, Trace=20, Stale=10
    let score2 = health_engine.calculate_health("proj_1", 0.0, 100.0, 0.0, 0.0);
    assert_eq!(score2.total_health, 35.0);
}

#[tokio::test]
async fn cert_06_coverage_snapshot() {
    let (store, _dir) = test_store();
    let repo = CoverageSnapshotRepository::new(store.clone());
    repo.setup_schema().unwrap();

    let snapshot = CoverageSnapshot {
        id: "snap_1".to_string(),
        project_id: "proj_1".to_string(),
        timestamp: chrono::Utc::now(),
        metrics: ares_completeness::models::CoverageMetrics {
            requirement_coverage: 100.0,
            decision_coverage: 90.0,
            architecture_coverage: 80.0,
            code_coverage: 70.0,
            test_coverage: 60.0,
            runtime_coverage: 50.0,
        },
        overall_coverage: 75.0,
    };

    repo.record_snapshot(&snapshot).unwrap();

    let snaps = repo.get_snapshots_for_project("proj_1").unwrap();
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].id, "snap_1");
    assert_eq!(snaps[0].metrics.requirement_coverage, 100.0);
}

#[tokio::test]
async fn cert_07_topology_determinism() {
    let (store1, _dir1) = test_store();
    setup_test_data(&store1);

    let (store2, _dir2) = test_store();
    let conn = store2.get_conn().unwrap();
    // Setup identical data but with reversed edge insertion order
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES ('proj_1', 'P1', 'mature', '/p1', 'rust', 'mature', 'mature', 0, 0)",
        [],
    ).unwrap();

    let nodes = vec![
        ("R1", "requirement"),
        ("D1", "decision"),
        ("A1", "architecture"),
        ("C1", "file"),
        ("T1", "test"),
        ("RS1", "runtime_signal"),
        ("O1", "outcome"),
        ("R2", "requirement"),
        ("D2", "decision"),
        ("A2", "architecture"),
        ("R3", "requirement"),
    ];
    for (id, nt) in nodes {
        conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES (?1, 'proj_1', ?2, ?1, '{}', 0, 0)", rusqlite::params![id, nt]).unwrap();
    }

    let mut edges = vec![
        ("R1", "D1"),
        ("D1", "A1"),
        ("A1", "C1"),
        ("C1", "T1"),
        ("T1", "RS1"),
        ("RS1", "O1"),
        ("R2", "D2"),
        ("D2", "A2"),
    ];
    edges.reverse(); // Insert edges backwards

    for (edge_id, (from, to)) in (1..).zip(edges) {
        conn.execute(
            "INSERT INTO graph_edges (id, project_id, from_node_id, to_node_id, edge_type, valid_from, created_at) VALUES (?1, 'proj_1', ?2, ?3, 'evolves', 0, 0)",
            rusqlite::params![format!("e{}", edge_id), from, to],
        ).unwrap();
    }

    let topology_engine1 = TopologyEngine::new(store1.clone());
    let mut segments1 = topology_engine1
        .evaluate_topology(&"proj_1".into())
        .unwrap();
    segments1.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    let topology_engine2 = TopologyEngine::new(store2.clone());
    let mut segments2 = topology_engine2
        .evaluate_topology(&"proj_1".into())
        .unwrap();
    segments2.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    assert_eq!(segments1.len(), segments2.len());
    for i in 0..segments1.len() {
        assert_eq!(segments1[i].node_id, segments2[i].node_id);
        assert_eq!(segments1[i].state, segments2[i].state);
        assert_eq!(
            segments1[i].missing_downstream,
            segments2[i].missing_downstream
        );
    }
}
