use ares_core::id::NodeId;
use ares_core::types::evidence::{Evidence, EvidenceSource, EvidenceType};
use ares_evolution::EvidenceEngine;
use ares_store::db::test_helpers::test_store;
use ares_store::repositories::evidence::{EvidenceRepository, SqliteEvidenceRepository};
use std::sync::Arc;

#[tokio::test]
async fn test_extract_facts() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteEvidenceRepository::new(store));
    let engine = EvidenceEngine::new(repo);

    let source_node = NodeId::new();

    // Fact 1
    let evidence_dep = engine.extract_facts_from_content(
        source_node.clone(),
        "Cargo.toml",
        r#"
[dependencies]
oauth2 = "4.0"
        "#,
    );
    assert_eq!(evidence_dep.len(), 1);
    assert_eq!(evidence_dep[0].evidence_type, EvidenceType::DependencyFact);
    assert_eq!(evidence_dep[0].observed_value, "Uses OAuth2");

    // Fact 2
    let evidence_import = engine.extract_facts_from_content(
        source_node.clone(),
        "src/main.rs",
        "use oauth2::Config;",
    );
    assert_eq!(evidence_import.len(), 1);
    assert_eq!(evidence_import[0].evidence_type, EvidenceType::ScannerFact);
    assert_eq!(
        evidence_import[0].observed_value,
        "OAuth2 capability detected"
    );

    // Fact 3
    let evidence_conf =
        engine.extract_facts_from_content(source_node.clone(), ".env", "OIDC_ENABLED=true");
    assert_eq!(evidence_conf.len(), 1);
    assert_eq!(
        evidence_conf[0].evidence_type,
        EvidenceType::ConfigurationFact
    );
    assert_eq!(evidence_conf[0].observed_value, "OIDC capability detected");

    // Fact 4
    let evidence_own =
        engine.extract_facts_from_content(source_node.clone(), "CODEOWNERS", "* @team-auth");
    assert_eq!(evidence_own.len(), 1);
    assert_eq!(evidence_own[0].evidence_type, EvidenceType::OwnershipFact);
    assert_eq!(evidence_own[0].observed_value, "Ownership fact");
}

#[tokio::test]
async fn test_record_and_get_evidence() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteEvidenceRepository::new(store.clone()));
    let engine = EvidenceEngine::new(repo);

    let project_id = ares_core::id::new_id();
    let target_node = NodeId::new();

    // Satisfy foreign keys
    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES (?1, 'test', '', '/tmp', 'rust', '', 'mature', 0, 0)",
        rusqlite::params![project_id],
    ).unwrap();

    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES (?1, ?2, 'file', 'target', '{}', 0, 0)",
        rusqlite::params![target_node.as_str(), project_id],
    ).unwrap();

    let evidence = Evidence {
        id: NodeId::new(),
        evidence_type: EvidenceType::ScannerFact,
        source_node: target_node.clone(),
        observed_value: "OAuth2 capability detected".to_string(),
        observed_at: chrono::Utc::now(),
        confidence: 0.95,
        source: EvidenceSource::Scanner,
    };

    engine
        .record_evidence(&project_id, evidence.clone())
        .await
        .expect("Failed to record evidence");

    let retrieved = engine
        .get_evidence_for_node(&project_id, target_node.as_str())
        .await
        .expect("Failed to get evidence");
    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].id, evidence.id);
    assert_eq!(retrieved[0].observed_value, "OAuth2 capability detected");
}
