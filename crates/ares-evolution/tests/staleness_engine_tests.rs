use ares_core::types::evolution::EvolutionEventType;
use ares_core::types::staleness::{HealthClassification, StalenessFactors};
use ares_evolution::StalenessEngine;
use ares_store::db::test_helpers::test_store;
use ares_store::repositories::evolution::{EvolutionRepository, SqliteEvolutionRepository};
use std::sync::Arc;

#[tokio::test]
async fn test_age_increases_score_decreases() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));
    let engine = StalenessEngine::new(repo);

    let mut factors = StalenessFactors {
        age_days: 100,
        downstream_changes: 0,
        dependent_nodes: 0,
        ownership_changes: 0,
        evolution_events: 0,
    };

    let result_100 = engine
        .analyze("proj", "node", "decision", &factors, None)
        .await
        .unwrap()
        .unwrap();

    factors.age_days = 500;
    let result_500 = engine
        .analyze("proj", "node", "decision", &factors, None)
        .await
        .unwrap()
        .unwrap();

    assert!(result_100.score > result_500.score);
}

#[tokio::test]
async fn test_determinism() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));
    let engine = StalenessEngine::new(repo);

    let factors = StalenessFactors {
        age_days: 800,
        downstream_changes: 20, // high volatility
        dependent_nodes: 15,
        ownership_changes: 3,
        evolution_events: 0,
    };

    let result_1 = engine
        .analyze("proj", "node", "decision", &factors, None)
        .await
        .unwrap()
        .unwrap();

    for _ in 0..9 {
        let result_n = engine
            .analyze("proj", "node", "decision", &factors, None)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result_1.score, result_n.score);
        assert_eq!(result_1.classification, result_n.classification);
        assert_eq!(result_1.rationale, result_n.rationale);
    }

    // Should be exactly as expected in prompt example (around 36)
    assert!(result_1.score < 40.0);
    assert_eq!(result_1.classification, HealthClassification::Critical);
}

#[tokio::test]
async fn test_repository_isolation() {
    let (store, _dir) = test_store();

    // Project A
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
    // Project B
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES ('proj_b', 'test B', '', '/tmp/b', 'rust', '', 'mature', 0, 0)",
        [],
    ).unwrap();

    let repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));
    let engine = StalenessEngine::new(repo.clone());

    let factors = StalenessFactors {
        age_days: 800,
        downstream_changes: 20,
        dependent_nodes: 15,
        ownership_changes: 3,
        evolution_events: 0,
    };

    // Analyze proj_a and trigger an event because of classification drop
    engine
        .analyze(
            "proj_a",
            "node_a",
            "decision",
            &factors,
            Some(HealthClassification::Healthy),
        )
        .await
        .unwrap();

    // Verify proj_a has event, proj_b has no events
    let events_a = repo.get_events_for_node("proj_a", "node_a").await.unwrap();
    let events_b = repo.get_events_for_node("proj_b", "node_a").await.unwrap();

    assert_eq!(events_a.len(), 1);
    assert_eq!(
        events_a[0].event_type,
        EvolutionEventType::StalenessDetected
    );
    assert_eq!(events_b.len(), 0);
}

#[tokio::test]
async fn test_code_no_staleness() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));
    let engine = StalenessEngine::new(repo);

    let factors = StalenessFactors {
        age_days: 8000,
        downstream_changes: 200,
        dependent_nodes: 150,
        ownership_changes: 30,
        evolution_events: 0,
    };

    let result = engine
        .analyze("proj", "node", "code", &factors, None)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(result.score, 100.0);
    assert_eq!(result.classification, HealthClassification::Healthy);
    assert!(result.rationale[0].contains("Code cannot become stale"));
}

#[tokio::test]
async fn test_explainability() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));
    let engine = StalenessEngine::new(repo);

    let factors = StalenessFactors {
        age_days: 730,
        downstream_changes: 10,
        dependent_nodes: 12,
        ownership_changes: 2,
        evolution_events: 0,
    };

    let result = engine
        .analyze("proj", "node", "decision", &factors, None)
        .await
        .unwrap()
        .unwrap();

    // Check rationale includes all factors
    let rationale = result.rationale.join("\n");
    assert!(rationale.contains("Age: 730 days"));
    assert!(rationale.contains("Dependencies: 12"));
    assert!(rationale.contains("Ownership Changes: 2"));
    assert!(rationale.contains("Volatility: 10"));
    assert!(rationale.contains("Final Score:"));
}
