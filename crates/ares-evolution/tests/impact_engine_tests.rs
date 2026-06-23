use ares_core::types::drift::{DriftCandidate, DriftType};
use ares_core::types::impact::ImpactSeverity;
use ares_core::types::staleness::{HealthClassification, StalenessFinding};
use ares_evolution::MemoryImpactEngine;
use ares_store::db::test_helpers::test_store;
use ares_store::repositories::drift::{DriftRepository, SqliteDriftRepository};
use chrono::Utc;
use std::sync::Arc;

#[tokio::test]
async fn test_determinism() {
    let (store, _dir) = test_store();

    // Project A
    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES ('proj_a', 'test A', '', '/tmp/a', 'rust', '', 'mature', 0, 0)",
        [],
    ).unwrap();

    let repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let engine = MemoryImpactEngine::new(repo.clone());

    let findings = vec![StalenessFinding {
        node_id: "node_b".to_string(),
        project_id: "proj_a".to_string(),
        score: 20.0,
        classification: HealthClassification::Critical,
        rationale: vec![],
    }];

    let result_1 = engine
        .analyze_impact(
            "proj_a",
            "target",
            &[("node_b".to_string(), "decision".to_string())],
            vec![],
            vec![],
            &findings,
        )
        .await
        .unwrap();

    for _ in 0..9 {
        let result_n = engine
            .analyze_impact(
                "proj_a",
                "target",
                &[("node_b".to_string(), "decision".to_string())],
                vec![],
                vec![],
                &findings,
            )
            .await
            .unwrap();
        assert_eq!(result_1.total_impact_score, result_n.total_impact_score);
    }
}

#[tokio::test]
async fn test_repository_isolation() {
    let (store, _dir) = test_store();

    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES ('proj_a', 'test A', '', '/tmp/a', 'rust', '', 'mature', 0, 0)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES ('node_a', 'proj_a', 'decision', 'Decision A', '{}', 0, 0)",
        [],
    ).unwrap();

    let repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let engine = MemoryImpactEngine::new(repo.clone());

    // Create a drift candidate in proj_a
    repo.record_candidate(DriftCandidate {
        id: "drift_1".to_string(),
        project_id: "proj_a".to_string(),
        target_node_id: "node_a".to_string(),
        drift_type: DriftType::DependencyMismatch,
        confidence: 1.0,
        evidence_ids: vec![],
        rationale: "Rationale".to_string(),
        detected_at: Utc::now(),
    })
    .await
    .unwrap();

    // Query impact for proj_b hitting node_a
    let result_b = engine
        .analyze_impact(
            "proj_b", // Different project
            "target",
            &[("node_a".to_string(), "decision".to_string())],
            vec![],
            vec![],
            &[],
        )
        .await
        .unwrap();

    // Query impact for proj_a hitting node_a
    let result_a = engine
        .analyze_impact(
            "proj_a",
            "target",
            &[("node_a".to_string(), "decision".to_string())],
            vec![],
            vec![],
            &[],
        )
        .await
        .unwrap();

    assert_eq!(result_b.drift_risk, 0.0);
    assert_eq!(result_a.drift_risk, 25.0);
}

#[tokio::test]
async fn test_drift_and_staleness_integration() {
    let (store, _dir) = test_store();

    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES ('proj_a', 'test A', '', '/tmp/a', 'rust', '', 'mature', 0, 0)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES ('node_a', 'proj_a', 'requirement', 'Req A', '{}', 0, 0)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES ('node_b', 'proj_a', 'decision', 'Decision B', '{}', 0, 0)",
        [],
    ).unwrap();

    let repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let engine = MemoryImpactEngine::new(repo.clone());

    // Drift on node_a
    repo.record_candidate(DriftCandidate {
        id: "drift_1".to_string(),
        project_id: "proj_a".to_string(),
        target_node_id: "node_a".to_string(),
        drift_type: DriftType::DependencyMismatch,
        confidence: 1.0,
        evidence_ids: vec![],
        rationale: "Rationale".to_string(),
        detected_at: Utc::now(),
    })
    .await
    .unwrap();

    // Staleness on node_b
    let findings = vec![StalenessFinding {
        node_id: "node_b".to_string(),
        project_id: "proj_a".to_string(),
        score: 30.0,
        classification: HealthClassification::Critical,
        rationale: vec![],
    }];

    let result = engine
        .analyze_impact(
            "proj_a",
            "target",
            &[
                ("node_a".to_string(), "requirement".to_string()),
                ("node_b".to_string(), "decision".to_string()),
            ],
            vec![],
            vec![],
            &findings,
        )
        .await
        .unwrap();

    // Drift risk should be 25.0 (1 matched drift)
    assert_eq!(result.drift_risk, 25.0);
    // Staleness risk should be 100.0 - 30.0 = 70.0
    assert_eq!(result.staleness_risk, 70.0);

    // Structural: 1 Req (30) + 1 Dec (20) = 50.0
    // Score: (50 * 0.4) + (25 * 0.3) + (70 * 0.3) = 20 + 7.5 + 21 = 48.5
    assert_eq!(result.total_impact_score, 48.5);
    assert_eq!(result.severity, ImpactSeverity::Medium);
}

#[tokio::test]
async fn test_explainability() {
    let (store, _dir) = test_store();
    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES ('proj_a', 'test A', '', '/tmp/a', 'rust', '', 'mature', 0, 0)",
        [],
    ).unwrap();

    let repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let engine = MemoryImpactEngine::new(repo.clone());

    let result = engine
        .analyze_impact(
            "proj_a",
            "target",
            &[("node_a".to_string(), "requirement".to_string())],
            vec![],
            vec![],
            &[],
        )
        .await
        .unwrap();

    assert!(result
        .rationale
        .iter()
        .any(|r| r.contains("Total Impact Score")));
}
