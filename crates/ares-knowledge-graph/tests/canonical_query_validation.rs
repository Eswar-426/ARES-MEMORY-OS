use ares_knowledge_graph::impact::ImpactEngine;
use ares_knowledge_graph::models::{DomainEvent, DomainEventType, ProjectionMetrics};
use ares_knowledge_graph::projection::{ProjectionEngine, ProjectionMode};
use ares_knowledge_graph::projector_registry::{
    DecisionProjector, GapProjector, ProjectorRegistry, RequirementProjector, ResolutionProjector,
};
use ares_knowledge_graph::queries::CanonicalQueries;
use ares_knowledge_graph::store::KnowledgeGraphStore;
use ares_knowledge_graph::traversal::TraversalEngine;
use ares_store::Store;
use rusqlite::params;
use std::sync::Arc;
use tempfile::tempdir;

async fn setup_test_engine() -> (
    Arc<Store>,
    Arc<KnowledgeGraphStore>,
    ProjectionEngine,
    ProjectorRegistry,
    CanonicalQueries,
) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_queries.db");
    let store = Arc::new(Store::open(&db_path).unwrap());
    let kg_store = Arc::new(KnowledgeGraphStore::new(store.clone()));

    let engine = ProjectionEngine::new(kg_store.clone());

    let mut registry = ProjectorRegistry::new();
    registry.register(Box::new(RequirementProjector));
    registry.register(Box::new(DecisionProjector));
    registry.register(Box::new(GapProjector));
    registry.register(Box::new(ResolutionProjector));

    let traversal = Arc::new(TraversalEngine::new(kg_store.clone()));
    let impact = Arc::new(ImpactEngine::new(traversal.clone()));
    let queries = CanonicalQueries::new(traversal.clone(), impact.clone());

    (store, kg_store, engine, registry, queries)
}

async fn seed_authentication_graph(
    kg_store: &Arc<KnowledgeGraphStore>,
) {
    use ares_knowledge_graph::models::{KnowledgeNode, KnowledgeEdge, NodeType, EdgeType};
    use serde_json::json;

    let nodes = vec![
        ("REQ-AUTH", NodeType::Requirement, "REQ-AUTH"),
        ("DEC-JWT", NodeType::Decision, "DEC-JWT"),
        ("GAP-AUTH", NodeType::Gap, "GAP-AUTH"),
        ("RES-ROTATION", NodeType::Resolution, "RES-ROTATION"),
        ("OWNER-ALICE", NodeType::Owner, "Alice"),
    ];
    for (id, node_type, name) in nodes {
        kg_store.upsert_node(&KnowledgeNode {
            id: id.to_string(),
            node_type,
            name: name.to_string(),
            properties: json!({}),
            created_at: 1000,
        }).unwrap();
    }
    
    // Insert edges
    let edges = vec![
        ("EDGE-REQ-DEC", "REQ-AUTH", "DEC-JWT", EdgeType::Drives),
        ("EDGE-GAP-DEC", "GAP-AUTH", "DEC-JWT", EdgeType::Causes),
        ("EDGE-DEC-GAP", "DEC-JWT", "GAP-AUTH", EdgeType::Causes),
        ("EDGE-GAP-RES", "GAP-AUTH", "RES-ROTATION", EdgeType::Causes),
        ("EDGE-OWNER-DEC", "DEC-JWT", "OWNER-ALICE", EdgeType::ApprovedBy),
    ];
    for (id, source, target, edge_type) in edges {
        kg_store.upsert_edge(&KnowledgeEdge {
            id: id.to_string(),
            source_id: source.to_string(),
            target_id: target.to_string(),
            edge_type,
            confidence: 1.0,
            created_at: 1000,
            properties: json!({}),
        }).unwrap();
    }
}

#[tokio::test]
async fn test_canonical_queries() {
    let (store, kg_store, engine, registry, queries) = setup_test_engine().await;

    seed_authentication_graph(&kg_store).await;

    // 1. Why does authentication exist? (Start from DEC-JWT and go upstream)
    let why_result = queries.why_does_this_exist("DEC-JWT").unwrap();
    assert_eq!(why_result.requirements.len(), 1);
    assert_eq!(why_result.requirements[0].id, "REQ-AUTH");

    // 2. Who owns authentication? (Start from DEC-JWT and go upstream to Owner)
    let ownership_result = queries.who_owns_this("DEC-JWT").unwrap();
    assert_eq!(ownership_result.owners.len(), 1);
    assert_eq!(ownership_result.owners[0].name, "Alice");

    // 3. What breaks if authentication changes? (Start from REQ-AUTH and go downstream)
    let impact_result = queries.what_breaks_if_changed("REQ-AUTH").unwrap();
    assert!(impact_result.total_score > 0);

    // 4. What debt surrounds authentication?
    let debt_result = queries.what_knowledge_debt_exists("DEC-JWT").unwrap();
    assert_eq!(debt_result.gaps.len(), 1);
    assert_eq!(debt_result.gaps[0].id, "GAP-AUTH");
    assert_eq!(debt_result.resolutions.len(), 1);
    assert_eq!(debt_result.resolutions[0].id, "RES-ROTATION");
}
