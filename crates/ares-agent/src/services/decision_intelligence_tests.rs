#[cfg(test)]
mod tests {
    use crate::services::decision_intelligence::DecisionIntelligenceEngine;
    use ares_core::id::new_id;
    use ares_core::{
        CreateDecisionInput, CreateMemoryInput, EdgeType, GraphEdge, GraphNode, MemoryType, NodeId,
        NodeType, ProjectId,
    };
    use ares_store::repositories::decision::SqliteDecisionRepository;
    use ares_store::repositories::graph::SqliteGraphRepository;
    use ares_store::repositories::memory::SqliteMemoryRepository;
    use ares_store::repositories::project::SqliteProjectRepository;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn test_store() -> (ares_store::db::Store, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = ares_store::db::Store::open(&dir.path().join("test.db")).unwrap();
        (store, dir)
    }

    #[test]
    fn test_decision_lineage() {
        let (store, _dir) = test_store();

        // Setup repositories
        let decision_repo = Arc::new(SqliteDecisionRepository::new(store.clone()));
        let graph_repo = Arc::new(SqliteGraphRepository::new(store.clone()));
        let engine = DecisionIntelligenceEngine::new(decision_repo.clone(), graph_repo.clone());

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

        let memory_repo = SqliteMemoryRepository::new(store.clone());
        let memory = memory_repo
            .create(CreateMemoryInput {
                project_id: project_id.clone(),
                memory_type: MemoryType::Decision,
                title: "Test Memory".to_string(),
                content: serde_json::json!({}),
                confidence: None,
                importance: None,
                source: None,
                ai_assisted: None,
            })
            .unwrap();

        // Create initial decision A
        let dec_a = decision_repo
            .create(CreateDecisionInput {
                project_id: project_id.clone(),
                title: "Use REST".to_string(),
                memory_id: memory.id.clone(),
                decision_text: "REST API".to_string(),
                reason: "Initial API".to_string(),
                confidence: None,
                alternatives: None,
                risks: None,
                context_snapshot: None,
                future_impact: None,
                files_impacted: None,
                services_impacted: None,
                supersedes: None,
                decided_by: None,
                discussed_in: None,
                review_due_at: None,
            })
            .unwrap();

        // Create decision B superseding A
        let dec_b = decision_repo
            .create(CreateDecisionInput {
                project_id: project_id.clone(),
                title: "Use GraphQL".to_string(),
                memory_id: memory.id.clone(),
                decision_text: "GraphQL API".to_string(),
                reason: "Need flexibility".to_string(),
                confidence: None,
                alternatives: None,
                risks: None,
                context_snapshot: None,
                future_impact: None,
                files_impacted: None,
                services_impacted: None,
                supersedes: None,
                decided_by: None,
                discussed_in: None,
                review_due_at: None,
            })
            .unwrap();

        // Create decision C derived from B
        let dec_c = decision_repo
            .create(CreateDecisionInput {
                project_id: project_id.clone(),
                title: "Use Apollo Client".to_string(),
                memory_id: memory.id.clone(),
                decision_text: "Apollo".to_string(),
                reason: "Frontend needs to query GraphQL".to_string(),
                confidence: None,
                alternatives: None,
                risks: None,
                context_snapshot: None,
                future_impact: None,
                files_impacted: None,
                services_impacted: None,
                supersedes: None,
                decided_by: None,
                discussed_in: None,
                review_due_at: None,
            })
            .unwrap();

        // Insert Graph Nodes for the decisions
        let node_a = GraphNode {
            id: NodeId::from(dec_a.id.as_str()),
            project_id: project_id.clone(),
            node_type: NodeType::Decision,
            label: dec_a.title.clone(),
            properties: serde_json::json!({}),
            file_path: None,
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        };
        graph_repo.upsert_node(node_a).unwrap();

        let node_b = GraphNode {
            id: NodeId::from(dec_b.id.as_str()),
            project_id: project_id.clone(),
            node_type: NodeType::Decision,
            label: dec_b.title.clone(),
            properties: serde_json::json!({}),
            file_path: None,
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        };
        graph_repo.upsert_node(node_b).unwrap();

        let node_c = GraphNode {
            id: NodeId::from(dec_c.id.as_str()),
            project_id: project_id.clone(),
            node_type: NodeType::Decision,
            label: dec_c.title.clone(),
            properties: serde_json::json!({}),
            file_path: None,
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        };
        graph_repo.upsert_node(node_c).unwrap();

        // Add Graph Edges
        let edge_ab = GraphEdge {
            id: new_id(),
            project_id: project_id.clone(),
            from_node_id: NodeId::from(dec_b.id.as_str()),
            to_node_id: NodeId::from(dec_a.id.as_str()),
            edge_type: EdgeType::Supersedes,
            weight: 1.0,
            confidence: 1.0,
            source: "agent".to_string(),
            valid_from: 0,
            valid_until: None,
            created_at: 0,
        };
        graph_repo.upsert_edge(edge_ab).unwrap();

        let edge_cb = GraphEdge {
            id: new_id(),
            project_id: project_id.clone(),
            from_node_id: NodeId::from(dec_c.id.as_str()),
            to_node_id: NodeId::from(dec_b.id.as_str()),
            edge_type: EdgeType::DerivedFrom,
            weight: 1.0,
            confidence: 1.0,
            source: "agent".to_string(),
            valid_from: 0,
            valid_until: None,
            created_at: 0,
        };
        graph_repo.upsert_edge(edge_cb).unwrap();

        // Test lineage queries
        let replaced = engine.what_replaced_this(&project_id, &dec_a.id).unwrap();
        assert!(replaced.is_some());
        assert_eq!(replaced.unwrap().id, dec_b.id);

        let evolved = engine
            .what_evolved_from_this(&project_id, &dec_b.id)
            .unwrap();
        assert_eq!(evolved.len(), 1);
        assert_eq!(evolved[0].id, dec_c.id);

        let history = engine.decision_history(&project_id, &dec_a.id).unwrap();
        assert_eq!(history.len(), 2); // A -> B
        assert_eq!(history[0].id, dec_a.id);
        assert_eq!(history[1].id, dec_b.id);
    }
}
