#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_helpers::test_store;
    use ares_core::{NodeType, NodeId, ProjectId};
    use ares_core::types::event::now_micros;
    use uuid::Uuid;

    fn create_candidate(id: &str, project_id: &str) -> Candidate {
        Candidate {
            id: id.to_string(),
            project_id: project_id.to_string(),
            title: "Test Candidate".to_string(),
            description: "description".to_string(),
            candidate_type: CandidateType::Requirement,
            decision_category: None,
            architecture_category: None,
            traceability_category: None,
            source_endpoint: None,
            target_endpoint: None,
            traceability_strength: None,
            ownership_domains: Vec::new(),
            dependent_components: Vec::new(),
            status: CandidateStatus::Proposed,
            confidence: CandidateConfidence {
                evidence_count: 2,
                source_diversity: 1,
                temporal_consistency: 0.8,
                cluster_strength: 0.9,
            },
            bootstrap_metadata: None,
            created_at: now_micros(),
            updated_at: now_micros(),
        }
    }

    #[tokio::test]
    async fn test_candidate_creation() {
        let (store, _dir) = test_store();
        
        // Ensure project exists
        let conn = store.get_conn().unwrap();
        conn.execute("INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('repo-a', 'repo-a', '', '', '', '', 'greenfield', 0, 0)", []).unwrap();
        
        let repo = SqliteCandidateRepository::new(store.clone());

        let candidate = create_candidate("cand-1", "repo-a");
        repo.insert_candidate(&candidate).await.unwrap();

        let fetched = repo.get_candidate("repo-a", "cand-1").await.unwrap().unwrap();
        assert_eq!(fetched.status, CandidateStatus::Proposed);

        // Verify NOT stored in graph_nodes
        let mut stmt = conn.prepare("SELECT count(*) FROM graph_nodes WHERE id = 'cand-1'").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_candidate_rejection() {
        let (store, _dir) = test_store();
        let conn = store.get_conn().unwrap();
        conn.execute("INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('repo-a', 'repo-a', '', '', '', '', 'greenfield', 0, 0)", []).unwrap();

        let repo = SqliteCandidateRepository::new(store.clone());

        let mut candidate = create_candidate("cand-2", "repo-a");
        repo.insert_candidate(&candidate).await.unwrap();

        // Reject
        candidate.status = CandidateStatus::Rejected;
        repo.update_candidate(&candidate).await.unwrap();

        let fetched = repo.get_candidate("repo-a", "cand-2").await.unwrap().unwrap();
        assert_eq!(fetched.status, CandidateStatus::Rejected);

        // Verify no node and no promotion
        let promotion = repo.get_promotion("repo-a", "cand-2").await.unwrap();
        assert!(promotion.is_none());

        let mut stmt = conn.prepare("SELECT count(*) FROM graph_nodes").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_candidate_acceptance_and_isolation() {
        let (store, _dir) = test_store();
        let conn = store.get_conn().unwrap();
        conn.execute("INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('repo-a', 'repo-a', '', '', '', '', 'greenfield', 0, 0)", []).unwrap();
        conn.execute("INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('repo-b', 'repo-b', '', '', '', '', 'greenfield', 0, 0)", []).unwrap();

        let repo = SqliteCandidateRepository::new(store.clone());

        let candidate = create_candidate("cand-3", "repo-a");
        repo.insert_candidate(&candidate).await.unwrap();

        let node_id = NodeId::from("node-1".to_string());
        
        let mut node = GraphNode {
            id: node_id.clone(),
            project_id: ProjectId::from("repo-b".to_string()), // Intentional mismatch
            node_type: NodeType::Requirement,
            label: "Test Node".to_string(),
            properties: serde_json::json!({}),
            file_path: None,
            created_at: now_micros(),
            updated_at: now_micros(),
            deleted_at: None,
        };

        let promotion = CandidatePromotion {
            id: Uuid::new_v4().to_string(),
            candidate_id: "cand-3".to_string(),
            promoted_node_id: node_id.clone(),
            promoted_by: "test_user".to_string(),
            promoted_at: now_micros(),
            promotion_reason: None,
        };

        let source = CandidateSource {
            id: Uuid::new_v4().to_string(),
            candidate_id: "cand-3".to_string(),
            source_type: "commit".to_string(),
            source_id: "1234567".to_string(),
            confidence: 1.0,
        };
        repo.insert_source(&source).await.unwrap();

        // ISOLATION TEST: repo-a candidate, repo-b node should fail
        let result = repo.promote_candidate(&candidate, &promotion, &node, &[]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Repository mismatch"));

        // Now fix the node to matching repository
        node.project_id = ProjectId::from("repo-a".to_string());

        // ACCEPTANCE TEST
        repo.promote_candidate(&candidate, &promotion, &node, &[]).await.unwrap();

        // Verify Candidate Status
        let fetched = repo.get_candidate("repo-a", "cand-3").await.unwrap().unwrap();
        assert_eq!(fetched.status, CandidateStatus::Approved);

        // Verify Promotion Record
        let fetched_promo = repo.get_promotion("repo-a", "cand-3").await.unwrap().unwrap();
        assert_eq!(fetched_promo.promoted_node_id, node_id);
        assert_eq!(fetched_promo.promoted_by, "test_user");

        // Verify Node Created
        let mut stmt = conn.prepare("SELECT count(*) FROM graph_nodes WHERE id = 'node-1'").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_transaction_failure() {
        let (store, _dir) = test_store();
        let conn = store.get_conn().unwrap();
        conn.execute("INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('repo-a', 'repo-a', '', '', '', '', 'greenfield', 0, 0)", []).unwrap();

        let repo = SqliteCandidateRepository::new(store.clone());

        let candidate = create_candidate("cand-4", "repo-a");
        repo.insert_candidate(&candidate).await.unwrap();

        let node_id = NodeId::from("node-2".to_string());
        
        let node = GraphNode {
            id: node_id.clone(),
            project_id: ProjectId::from("INVALID-REPO-DOESNT-EXIST".to_string()), // Will fail foreign key constraint
            node_type: NodeType::Requirement,
            label: "Test Node".to_string(),
            properties: serde_json::json!({}),
            file_path: None,
            created_at: now_micros(),
            updated_at: now_micros(),
            deleted_at: None,
        };

        let promotion = CandidatePromotion {
            id: Uuid::new_v4().to_string(),
            candidate_id: "cand-4".to_string(),
            promoted_node_id: node_id.clone(),
            promoted_by: "test_user".to_string(),
            promoted_at: now_micros(),
            promotion_reason: None,
        };

        let source = CandidateSource {
            id: Uuid::new_v4().to_string(),
            candidate_id: "cand-4".to_string(),
            source_type: "commit".to_string(),
            source_id: "89abcdef".to_string(),
            confidence: 1.0,
        };
        repo.insert_source(&source).await.unwrap();

        // We temporarily bypass the application-layer check to force a DB constraint failure
        // We do this by faking candidate.project_id
        let mut fake_candidate = candidate.clone();
        fake_candidate.project_id = "INVALID-REPO-DOESNT-EXIST".to_string();

        let result = repo.promote_candidate(&fake_candidate, &promotion, &node, &[]).await;
        
        // Must fail
        assert!(result.is_err());

        // TRANSACTION VERIFICATION: No partial state
        let fetched = repo.get_candidate("repo-a", "cand-4").await.unwrap().unwrap();
        assert_eq!(fetched.status, CandidateStatus::Proposed); // Was not updated to Approved

        let fetched_promo = repo.get_promotion("repo-a", "cand-4").await.unwrap();
        assert!(fetched_promo.is_none());

        let mut stmt = conn.prepare("SELECT count(*) FROM graph_nodes WHERE id = 'node-2'").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}
