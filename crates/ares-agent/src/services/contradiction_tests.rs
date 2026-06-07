#[cfg(test)]
mod tests {
    use crate::services::contradiction_detector::ContradictionDetector;
    use ares_core::{GraphNode, NodeId, NodeType, ProjectId};
    use ares_store::repositories::graph::SqliteGraphRepository;
    use ares_store::repositories::intelligence::SqliteIntelligenceRepository;
    use ares_store::repositories::project::SqliteProjectRepository;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn test_store() -> (ares_store::db::Store, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = ares_store::db::Store::open(&dir.path().join("test.db")).unwrap();
        (store, dir)
    }

    #[test]
    fn test_contradiction_detection() {
        let (store, _dir) = test_store();

        let graph_repo = Arc::new(SqliteGraphRepository::new(store.clone()));
        let intell_repo = Arc::new(SqliteIntelligenceRepository::new(store.clone()));
        let detector = ContradictionDetector::new(graph_repo.clone(), intell_repo.clone());

        let project_id = ProjectId::new();
        let project_repo = SqliteProjectRepository::new(store.clone());
        project_repo
            .create(&ares_core::Project {
                id: project_id.clone(),
                name: "test".to_string(),
                description: "".to_string(),
                root_path: "/tmp/test".to_string(),
                primary_language: "rust".to_string(),
                domain: "".to_string(),
                maturity: ares_core::ProjectMaturity::Greenfield,
                created_at: 0,
                updated_at: 0,
                deleted_at: None,
            })
            .unwrap();

        // Node A
        let node_a = GraphNode {
            id: NodeId::new(),
            project_id: project_id.clone(),
            node_type: NodeType::Decision,
            label: "Auth Approach".to_string(),
            properties: serde_json::json!({"method": "JWT"}),
            file_path: None,
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        };
        graph_repo.upsert_node(node_a.clone()).unwrap();

        // Node B (Same label, different property -> contradiction)
        let node_b = GraphNode {
            id: NodeId::new(),
            project_id: project_id.clone(),
            node_type: NodeType::Decision,
            label: "Auth Approach".to_string(),
            properties: serde_json::json!({"method": "Session"}),
            file_path: None,
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        };
        graph_repo.upsert_node(node_b.clone()).unwrap();

        let start = std::time::Instant::now();
        let results = detector
            .detect_contradictions(&project_id, &[node_a.id.clone(), node_b.id.clone()])
            .unwrap();
        let duration = start.elapsed();

        println!("Contradiction detection took: {:?}", duration);
        assert!(
            duration.as_millis() < 1000,
            "Performance target: < 1 second"
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_id, node_a.id);
        assert_eq!(results[0].target_id, node_b.id);

        // Ensure edge was created
        let edges = graph_repo.get_edges_from(&node_a.id).unwrap();
        assert!(edges
            .iter()
            .any(|e| e.edge_type == ares_core::EdgeType::Contradicts));
    }
}
