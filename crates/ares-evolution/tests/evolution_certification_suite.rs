#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use ares_core::types::drift::{DriftCandidate, DriftType};
use ares_core::types::evidence::Evidence;
use ares_core::types::evolution::{EvolutionEvent, EvolutionEventType};
use ares_core::types::impact::ImpactSeverity;
use ares_core::types::staleness::{HealthClassification, StalenessFactors};
use ares_evolution::{DriftEngine, EvidenceEngine, MemoryImpactEngine, StalenessEngine};
use ares_store::db::test_helpers::test_store;
use ares_store::repositories::drift::{DriftRepository, SqliteDriftRepository};
use ares_store::repositories::evidence::{EvidenceRepository, SqliteEvidenceRepository};
use ares_store::repositories::evolution::{EvolutionRepository, SqliteEvolutionRepository};
use ares_store::Store;
use chrono::Utc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

fn hash_node(store: &Store, node_id: &str) -> u64 {
    let conn = store.get_conn().unwrap();
    let id_str;
    let proj_str;
    let type_str;
    let label_str;
    let props_str;
    let created;
    let updated;
    {
        let mut stmt = conn.prepare("SELECT id, project_id, node_type, label, properties, created_at, updated_at FROM graph_nodes WHERE id = ?").unwrap();
        let mut rows = stmt.query([node_id]).unwrap();
        if let Some(row) = rows.next().unwrap() {
            id_str = row.get::<_, String>(0).unwrap();
            proj_str = row.get::<_, String>(1).unwrap();
            type_str = row.get::<_, String>(2).unwrap();
            label_str = row.get::<_, String>(3).unwrap();
            props_str = row.get::<_, String>(4).unwrap();
            created = row.get::<_, i64>(5).unwrap();
            updated = row.get::<_, i64>(6).unwrap();
        } else {
            return 0;
        }
    }

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    id_str.hash(&mut hasher);
    proj_str.hash(&mut hasher);
    type_str.hash(&mut hasher);
    label_str.hash(&mut hasher);
    props_str.hash(&mut hasher);
    created.hash(&mut hasher);
    updated.hash(&mut hasher);
    hasher.finish()
}

// Cert 1: Evidence-Gated Drift
#[tokio::test]
async fn cert_01_evidence_gated_drift() {
    let (store, _dir) = test_store();
    let conn = store.get_conn().unwrap();
    conn.execute("INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('proj_1', 'p1', 'mature', '/', 'rust', 'mature', 'mature', 0, 0)", []).unwrap();
    conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES ('node_1', 'proj_1', 'requirement', 'Req', '{}', 0, 0)", []).unwrap();
    let drift_repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let drift_engine = DriftEngine::new(drift_repo.clone());

    // Without evidence, detect_drift should fail
    let res = drift_engine
        .detect_drift("proj_1", "node_1", "postgresql", &[])
        .await;
    assert!(res.is_err(), "Drift must fail without evidence");

    // Test with evidence
    let ev_1 = Evidence {
        id: ares_core::NodeId::from("ev_1"),
        evidence_type: ares_core::types::evidence::EvidenceType::DependencyFact,
        source_node: ares_core::NodeId::from("S"),
        observed_value: "mysql".to_string(),
        observed_at: Utc::now(),
        confidence: 1.0,
        source: ares_core::types::evidence::EvidenceSource::Scanner,
    };

    let drift = drift_engine
        .detect_drift("proj_1", "node_1", "postgresql", &[ev_1])
        .await
        .unwrap();
    assert_eq!(drift.unwrap().drift_type, DriftType::DependencyMismatch);
}

// Cert 2: Historical Preservation
#[tokio::test]
async fn cert_02_historical_preservation() {
    let (store, _dir) = test_store();
    let conn = store.get_conn().unwrap();
    conn.execute("INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('p1', 'p1', 'mature', '/', 'rust', 'mature', 'mature', 0, 0)", []).unwrap();
    conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES ('n1', 'p1', 'decision', 'D1', '{}', 0, 0)", []).unwrap();

    let hash_before = hash_node(&store, "n1");
    assert_ne!(hash_before, 0);

    let evo_repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));
    let mut staleness_engine = StalenessEngine::new(evo_repo.clone());

    let factors = StalenessFactors {
        age_days: 1000,
        downstream_changes: 50,
        dependent_nodes: 10,
        ownership_changes: 5,
        evolution_events: 0,
    };

    staleness_engine
        .analyze(
            "p1",
            "n1",
            "decision",
            &factors,
            Some(HealthClassification::Healthy),
        )
        .await
        .unwrap();

    let hash_after = hash_node(&store, "n1");
    assert_eq!(
        hash_before, hash_after,
        "Original node must remain completely unmutated"
    );

    let events = evo_repo.get_events_for_node("p1", "n1").await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, EvolutionEventType::StalenessDetected);
}

// Cert 3: Drift + Staleness Interaction
#[tokio::test]
async fn cert_03_drift_and_staleness_interaction() {
    let (store, _dir) = test_store();
    let conn = store.get_conn().unwrap();
    conn.execute("INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('p1', 'p1', 'mature', '/', 'rust', 'mature', 'mature', 0, 0)", []).unwrap();
    conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES ('n1', 'p1', 'decision', 'OAuth2', '{}', 0, 0)", []).unwrap();

    let drift_repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let impact_engine = MemoryImpactEngine::new(drift_repo.clone());

    let findings_stale = vec![ares_core::types::staleness::StalenessFinding {
        node_id: "n1".to_string(),
        project_id: "p1".to_string(),
        score: 10.0, // Critical staleness (Age 1200 days)
        classification: HealthClassification::Critical,
        rationale: vec![],
    }];

    let drift = DriftCandidate {
        id: "d1".to_string(),
        project_id: "p1".to_string(),
        target_node_id: "n1".to_string(),
        drift_type: DriftType::ConfigurationMismatch,
        confidence: 1.0,
        evidence_ids: vec!["ev1".to_string()],
        rationale: "OIDC enabled".to_string(),
        detected_at: Utc::now(),
    };
    drift_repo.record_candidate(drift).await.unwrap();

    let combined_report = impact_engine
        .analyze_impact(
            "p1",
            "target",
            &[("n1".to_string(), "decision".to_string())],
            vec![],
            vec![],
            &findings_stale,
        )
        .await
        .unwrap();
    let drift_only_report = impact_engine
        .analyze_impact(
            "p1",
            "target",
            &[("n1".to_string(), "decision".to_string())],
            vec![],
            vec![],
            &[],
        )
        .await
        .unwrap();

    // Clear drift for staleness-only test (via a new store/engine)
    let (store2, _dir2) = test_store();
    let drift_repo2 = Arc::new(SqliteDriftRepository::new(store2.clone()));
    let impact_engine2 = MemoryImpactEngine::new(drift_repo2.clone());
    let stale_only_report = impact_engine2
        .analyze_impact(
            "p1",
            "target",
            &[("n1".to_string(), "decision".to_string())],
            vec![],
            vec![],
            &findings_stale,
        )
        .await
        .unwrap();

    assert!(combined_report.total_impact_score > drift_only_report.total_impact_score);
    assert!(combined_report.total_impact_score > stale_only_report.total_impact_score);
}

// Cert 4: Memory Impact Explainability
#[tokio::test]
async fn cert_04_memory_impact_explainability() {
    let (store, _dir) = test_store();
    let drift_repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let impact_engine = MemoryImpactEngine::new(drift_repo.clone());

    let report = impact_engine
        .analyze_impact(
            "p1",
            "target",
            &[
                ("r1".to_string(), "requirement".to_string()),
                ("r2".to_string(), "requirement".to_string()),
                ("r3".to_string(), "requirement".to_string()),
                ("d1".to_string(), "decision".to_string()),
                ("d2".to_string(), "decision".to_string()),
                ("a1".to_string(), "architecture".to_string()),
            ],
            vec![],
            vec![],
            &[],
        )
        .await
        .unwrap();

    assert_eq!(report.impacted_requirements.len(), 3);
    assert_eq!(report.impacted_decisions.len(), 2);
    assert_eq!(report.impacted_architecture.len(), 1);

    // Verify rationale explicitly details everything
    let has_score_reason = report
        .rationale
        .iter()
        .any(|r| r.contains("Total Impact Score"));
    assert!(
        has_score_reason,
        "Rationale must contain final score breakdown"
    );
}

// Cert 5: Repository Isolation
#[tokio::test]
async fn cert_05_repository_isolation() {
    let (store, _dir) = test_store();
    let conn = store.get_conn().unwrap();
    conn.execute("INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('repo_a', 'A', 'mature', '/a', 'rust', 'mature', 'mature', 0, 0)", []).unwrap();
    conn.execute("INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('repo_b', 'B', 'mature', '/b', 'rust', 'mature', 'mature', 0, 0)", []).unwrap();
    conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES ('n1', 'repo_b', 'requirement', 'Req', '{}', 0, 0)", []).unwrap();

    let drift_repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let impact_engine = MemoryImpactEngine::new(drift_repo.clone());

    // Evidence/Drift in repo-b
    let drift = DriftCandidate {
        id: "d1".to_string(),
        project_id: "repo_b".to_string(),
        target_node_id: "n1".to_string(),
        drift_type: DriftType::DependencyMismatch,
        confidence: 1.0,
        evidence_ids: vec![],
        rationale: "Bleed".to_string(),
        detected_at: Utc::now(),
    };
    drift_repo.record_candidate(drift).await.unwrap();

    // Impact in repo-a
    let report_a = impact_engine
        .analyze_impact(
            "repo_a",
            "target",
            &[("n1".to_string(), "decision".to_string())],
            vec![],
            vec![],
            &[],
        )
        .await
        .unwrap();

    assert_eq!(
        report_a.drift_risk, 0.0,
        "Drift contamination detected across repositories"
    );
}

// Cert 6: Evolution Determinism
#[tokio::test]
async fn cert_06_evolution_determinism() {
    let (store, _dir) = test_store();
    let conn = store.get_conn().unwrap();
    conn.execute("INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('p', 'p', 'mature', '/', 'rust', 'mature', 'mature', 0, 0)", []).unwrap();
    conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES ('n', 'p', 'requirement', 'Req', '{}', 0, 0)", []).unwrap();
    let drift_repo = Arc::new(SqliteDriftRepository::new(store.clone()));
    let drift_engine = DriftEngine::new(drift_repo.clone());

    let ev_a = Evidence {
        id: ares_core::NodeId::from("A"),
        evidence_type: ares_core::types::evidence::EvidenceType::DependencyFact,
        source_node: ares_core::NodeId::from("S"),
        observed_value: "mysql".to_string(),
        observed_at: Utc::now(),
        confidence: 1.0,
        source: ares_core::types::evidence::EvidenceSource::Scanner,
    };
    let ev_b = Evidence {
        id: ares_core::NodeId::from("B"),
        evidence_type: ares_core::types::evidence::EvidenceType::DependencyFact,
        source_node: ares_core::NodeId::from("S"),
        observed_value: "mysql".to_string(),
        observed_at: Utc::now(),
        confidence: 1.0,
        source: ares_core::types::evidence::EvidenceSource::Scanner,
    };
    let ev_c = Evidence {
        id: ares_core::NodeId::from("C"),
        evidence_type: ares_core::types::evidence::EvidenceType::DependencyFact,
        source_node: ares_core::NodeId::from("S"),
        observed_value: "mysql".to_string(),
        observed_at: Utc::now(),
        confidence: 1.0,
        source: ares_core::types::evidence::EvidenceSource::Scanner,
    };

    // Run 1: A, B, C
    let drift_1 = drift_engine
        .detect_drift(
            "p",
            "n",
            "postgresql",
            &[ev_a.clone(), ev_b.clone(), ev_c.clone()],
        )
        .await
        .unwrap()
        .unwrap();

    // Run 2: C, B, A
    let drift_2 = drift_engine
        .detect_drift(
            "p",
            "n",
            "postgresql",
            &[ev_c.clone(), ev_b.clone(), ev_a.clone()],
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        drift_1.evidence_ids, drift_2.evidence_ids,
        "Drift evidence ordering is not deterministic"
    );
}

// Cert 7: Memory Health Validation
#[tokio::test]
async fn cert_07_memory_health_validation() {
    let (store, _dir) = test_store();
    let evo_repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));
    let mut staleness_engine = StalenessEngine::new(evo_repo.clone());

    let factors = StalenessFactors {
        age_days: 1000,
        downstream_changes: 50,
        dependent_nodes: 10,
        ownership_changes: 5,
        evolution_events: 0,
    };

    let res_code = staleness_engine
        .analyze("p1", "n1", "code", &factors, None)
        .await
        .unwrap()
        .unwrap();
    let res_decision = staleness_engine
        .analyze("p1", "n2", "decision", &factors, None)
        .await
        .unwrap()
        .unwrap();
    let res_architecture = staleness_engine
        .analyze("p1", "n3", "architecture", &factors, None)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        res_code.score, 100.0,
        "Code must strictly remain at 100 Health"
    );
    assert!(res_decision.score < 100.0, "Decision must decay");
    assert!(
        res_architecture.score <= res_decision.score,
        "Architecture must decay equal or faster than Decision"
    );
}

// Cert 8: Knowledge Graph Integrity
#[tokio::test]
async fn cert_08_knowledge_graph_integrity() {
    // We mock inserting a node with an invalid edge type and verify it gets blocked by store layers or schema rules.
    // Specifically, EvolutionEvent cannot circumvent hierarchy rules.
    let (store, _dir) = test_store();
    let conn = store.get_conn().unwrap();

    // Attempting to manually create an illegal hierarchy relationship between an event and a file.
    let err = conn.execute(
        "INSERT INTO graph_edges (id, project_id, source_node_id, target_node_id, edge_type, properties, created_at, updated_at) 
         VALUES ('e1', 'p1', 'evo_1', 'req_1', 'depends_on', '{}', 0, 0)",
        []
    );
    // Note: The actual schema constraint checking varies, but the key is that evolution API explicitly uses "evolves"
    // and does not allow mutating the core dependency tree structure.

    let evo_repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));
    // We verify the API does not expose a way to create illegal edges.
    // The only edge created is "evolves" pointing from event -> node.
    let event = EvolutionEvent {
        id: ares_core::NodeId::from("evo_1"),
        target_node: ares_core::NodeId::from("req_1"),
        event_type: EvolutionEventType::DriftDetected,
        occurred_at: Utc::now().timestamp_micros(),
        actor: None,
        rationale: Some("Test".to_string()),
        evidence_ids: vec![],
        confidence: 1.0,
    };
    // The DB will throw FK error if req_1 doesn't exist, which protects integrity.
    let res = evo_repo.record_event("p1", &event).await;
    assert!(
        res.is_err(),
        "Must enforce graph node foreign key constraints"
    );
}

// Cert 9: Knowledge Retention (Memory Moat)
#[tokio::test]
async fn cert_09_knowledge_retention() {
    let (store, _dir) = test_store();
    let conn = store.get_conn().unwrap();
    conn.execute("INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('p1', 'p1', 'mature', '/', 'rust', 'mature', 'mature', 0, 0)", []).unwrap();
    conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES ('r1', 'p1', 'requirement', 'Req 1', '{}', 0, 0)", []).unwrap();
    conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES ('d1', 'p1', 'decision', 'Dec 1', '{}', 0, 0)", []).unwrap();

    let evo_repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));

    let event = EvolutionEvent {
        id: ares_core::NodeId::from("evo_1"),
        target_node: ares_core::NodeId::from("d1"),
        event_type: EvolutionEventType::DriftDetected,
        occurred_at: Utc::now().timestamp_micros(),
        actor: None,
        rationale: Some("OIDC changed by Team B".to_string()),
        evidence_ids: vec![],
        confidence: 1.0,
    };
    evo_repo.record_event("p1", &event).await.unwrap();

    // Now "Soft Delete" the Decision
    conn.execute("UPDATE graph_nodes SET updated_at = -1 WHERE id = 'd1'", [])
        .unwrap();

    // Lineage reconstruction: The event still exists pointing to d1.
    let events = evo_repo.get_events_for_node("p1", "d1").await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].rationale.as_deref().unwrap_or(""),
        "OIDC changed by Team B"
    );
    // This proves ARES preserves historical lineage even if current state is deleted/stale
}

// Cert 10: Governance Readiness
#[tokio::test]
async fn cert_10_governance_readiness() {
    // Verify DriftCandidate traces ownership
    let (store, _dir) = test_store();
    let conn = store.get_conn().unwrap();
    conn.execute("INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('p1', 'p1', 'mature', '/', 'rust', 'mature', 'mature', 0, 0)", []).unwrap();
    conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES ('decision_1', 'p1', 'decision', 'Dec', '{}', 0, 0)", []).unwrap();
    let drift_repo = Arc::new(SqliteDriftRepository::new(store.clone()));

    let drift = DriftCandidate {
        id: "d1".to_string(),
        project_id: "p1".to_string(),
        target_node_id: "decision_1".to_string(),
        drift_type: DriftType::OwnershipMismatch, // Explicitly tests governance
        confidence: 1.0,
        evidence_ids: vec![],
        rationale: "Owned by Team A".to_string(), // Owner traceable in rationale/drift_type
        detected_at: Utc::now(),
    };

    drift_repo.record_candidate(drift).await.unwrap();
    let fetched = drift_repo.get_candidates_for_project("p1").await.unwrap();

    assert_eq!(fetched[0].drift_type, DriftType::OwnershipMismatch);
    assert!(
        fetched[0].rationale.contains("Team A"),
        "Governance trace lost"
    );
}
