use ares_core::id::NodeId;
use ares_core::types::drift::DriftType;
use ares_core::types::evidence::{Evidence, EvidenceSource, EvidenceType};
use ares_evolution::DriftEngine;
use ares_store::db::test_helpers::test_store;
use ares_store::repositories::drift::{DriftRepository, SqliteDriftRepository};
use chrono::Utc;
use std::sync::Arc;

#[tokio::test]
async fn test_drift_without_evidence_fails() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteDriftRepository::new(store));
    let engine = DriftEngine::new(repo);

    let result = engine
        .detect_drift("proj_1", "node_1", "Decision says OAuth2 only", &[])
        .await;
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Drift without evidence is not permitted"
    );
}

#[tokio::test]
async fn test_dependency_drift() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let engine = DriftEngine::new(repo.clone());

    let project_id = ares_core::id::new_id();

    // Satisfy foreign keys
    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES (?1, 'test', '', '/tmp', 'rust', '', 'mature', 0, 0)",
        rusqlite::params![project_id],
    ).unwrap();

    let target_node = NodeId::new();
    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES (?1, ?2, 'decision', 'Decision: Use PostgreSQL', '{}', 0, 0)",
        rusqlite::params![target_node.as_str(), project_id],
    ).unwrap();

    let evidence = vec![Evidence {
        id: NodeId::new(),
        evidence_type: EvidenceType::DependencyFact,
        source_node: target_node.clone(),
        observed_value: "mysql detected".to_string(),
        observed_at: Utc::now(),
        confidence: 1.0,
        source: EvidenceSource::Scanner,
    }];

    let result = engine
        .detect_drift(
            &project_id,
            target_node.as_str(),
            "Decision: Use PostgreSQL",
            &evidence,
        )
        .await
        .unwrap();
    assert!(result.is_some());
    let candidate = result.unwrap();
    assert_eq!(candidate.drift_type, DriftType::DependencyMismatch);
    assert_eq!(candidate.evidence_ids.len(), 1);
    assert!(candidate
        .rationale
        .contains("MySQL detected but memory specifies PostgreSQL"));

    // Verify it was saved to the repository
    let saved_candidates = repo.get_candidates_for_project(&project_id).await.unwrap();
    assert_eq!(saved_candidates.len(), 1);
    assert_eq!(saved_candidates[0].id, candidate.id);
}

#[tokio::test]
async fn test_configuration_drift() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let engine = DriftEngine::new(repo);

    let project_id = ares_core::id::new_id();

    // Satisfy foreign keys
    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES (?1, 'test', '', '/tmp', 'rust', '', 'mature', 0, 0)",
        rusqlite::params![project_id],
    ).unwrap();

    let target_node = NodeId::new();
    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES (?1, ?2, 'decision', 'Decision: OAuth2', '{}', 0, 0)",
        rusqlite::params![target_node.as_str(), project_id],
    ).unwrap();

    let evidence = vec![Evidence {
        id: NodeId::new(),
        evidence_type: EvidenceType::ConfigurationFact,
        source_node: target_node.clone(),
        observed_value: "OIDC_ENABLED=true".to_string(),
        observed_at: Utc::now(),
        confidence: 1.0,
        source: EvidenceSource::Scanner,
    }];

    let result = engine
        .detect_drift(&project_id, target_node.as_str(), "OAuth2", &evidence)
        .await
        .unwrap();
    assert!(result.is_some());
    let candidate = result.unwrap();
    assert_eq!(candidate.drift_type, DriftType::ConfigurationMismatch);
}

#[tokio::test]
async fn test_ownership_drift() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let engine = DriftEngine::new(repo);

    let project_id = ares_core::id::new_id();

    // Satisfy foreign keys
    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES (?1, 'test', '', '/tmp', 'rust', '', 'mature', 0, 0)",
        rusqlite::params![project_id],
    ).unwrap();

    let target_node = NodeId::new();
    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES (?1, ?2, 'decision', 'Decision: Platform Team', '{}', 0, 0)",
        rusqlite::params![target_node.as_str(), project_id],
    ).unwrap();

    let evidence = vec![Evidence {
        id: NodeId::new(),
        evidence_type: EvidenceType::OwnershipFact,
        source_node: target_node.clone(),
        observed_value: "CODEOWNERS: Identity Team".to_string(),
        observed_at: Utc::now(),
        confidence: 1.0,
        source: EvidenceSource::Scanner,
    }];

    let result = engine
        .detect_drift(
            &project_id,
            target_node.as_str(),
            "Platform Team",
            &evidence,
        )
        .await
        .unwrap();
    assert!(result.is_some());
    let candidate = result.unwrap();
    assert_eq!(candidate.drift_type, DriftType::OwnershipMismatch);
}

#[tokio::test]
async fn test_repository_isolation() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let engine = DriftEngine::new(repo.clone());

    let proj_a = ares_core::id::new_id();
    let proj_b = ares_core::id::new_id();

    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES (?1, 'proj_a', '', '/tmp/a', 'rust', '', 'mature', 0, 0)",
        rusqlite::params![proj_a],
    ).unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES (?1, 'proj_b', '', '/tmp/b', 'rust', '', 'mature', 0, 0)",
        rusqlite::params![proj_b],
    ).unwrap();

    let target_node_a = NodeId::new();
    let target_node_b = NodeId::new();
    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES (?1, ?2, 'decision', 'Decision', '{}', 0, 0)",
        rusqlite::params![target_node_a.as_str(), proj_a],
    ).unwrap();
    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES (?1, ?2, 'decision', 'Decision', '{}', 0, 0)",
        rusqlite::params![target_node_b.as_str(), proj_b],
    ).unwrap();

    let evidence_a = vec![Evidence {
        id: NodeId::new(),
        evidence_type: EvidenceType::DependencyFact,
        source_node: target_node_a.clone(),
        observed_value: "mysql detected".to_string(),
        observed_at: Utc::now(),
        confidence: 1.0,
        source: EvidenceSource::Scanner,
    }];

    // Generate drift for A
    engine
        .detect_drift(&proj_a, target_node_a.as_str(), "PostgreSQL", &evidence_a)
        .await
        .unwrap();

    // Verify A has it, B does not
    let drifts_a = repo.get_candidates_for_project(&proj_a).await.unwrap();
    let drifts_b = repo.get_candidates_for_project(&proj_b).await.unwrap();

    assert_eq!(drifts_a.len(), 1);
    assert_eq!(drifts_b.len(), 0);
}

#[tokio::test]
async fn test_determinism() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let engine = DriftEngine::new(repo);

    let project_id = ares_core::id::new_id();
    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES (?1, 'test', '', '/tmp', 'rust', '', 'mature', 0, 0)",
        rusqlite::params![project_id],
    ).unwrap();

    let target_node = NodeId::new();
    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES (?1, ?2, 'decision', 'Decision: Use PostgreSQL', '{}', 0, 0)",
        rusqlite::params![target_node.as_str(), project_id],
    ).unwrap();

    let evidence = vec![Evidence {
        id: NodeId::new(),
        evidence_type: EvidenceType::DependencyFact,
        source_node: target_node.clone(),
        observed_value: "mysql detected".to_string(),
        observed_at: Utc::now(),
        confidence: 1.0,
        source: EvidenceSource::Scanner,
    }];

    let mut _first_id = String::new();

    // Run 10 times to ensure determinism in outcome
    for i in 0..10 {
        let result = engine
            .detect_drift(
                &project_id,
                target_node.as_str(),
                "Decision: Use PostgreSQL",
                &evidence,
            )
            .await
            .unwrap();
        let candidate = result.unwrap();

        if i == 0 {
            _first_id = candidate.id.clone();
        } else {
            // Because we create new IDs inside detect_drift, the candidate IDs will be different,
            // but the drift_type and rationale must be strictly identical.
            assert_eq!(candidate.drift_type, DriftType::DependencyMismatch);
            assert_eq!(candidate.rationale, "Observed: mysql detected\nMemory: Decision: Use PostgreSQL\nConflict: MySQL detected but memory specifies PostgreSQL");
        }
    }
}
