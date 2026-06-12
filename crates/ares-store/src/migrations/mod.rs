use ares_core::AresError;
use rusqlite::Connection;

// refinery requires migrations embedded at compile time.
// The macro scans the `migrations/` directory relative to this file.
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/migrations");
}

/// Run all pending migrations on the given connection.
pub fn run(conn: &mut Connection) -> Result<(), AresError> {
    embedded::migrations::runner()
        .run(conn)
        .map(|_report| ())
        .map_err(|e| AresError::migration(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_database_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        // Run migrations on a fresh connection
        let result = run(&mut conn);
        assert!(result.is_ok(), "Fresh migrations failed: {:?}", result);

        // Verify tables exist by querying sqlite_master
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"memories".to_string()));
        assert!(tables.contains(&"decisions".to_string()));
        assert!(tables.contains(&"graph_nodes".to_string()));
        assert!(tables.contains(&"graph_edges".to_string()));
        assert!(tables.contains(&"events".to_string()));
        assert!(tables.contains(&"workflow_visualizations".to_string()));
        assert!(tables.contains(&"workflow_analytics_cache".to_string()));
        assert!(tables.contains(&"replay_audit_log".to_string()));
        assert!(tables.contains(&"event_store".to_string()));
        assert!(tables.contains(&"event_aggregates".to_string()));
        assert!(tables.contains(&"event_snapshots".to_string()));
        assert!(tables.contains(&"event_consumer_groups".to_string()));
        assert!(tables.contains(&"event_group_offsets".to_string()));
        assert!(tables.contains(&"event_dlq".to_string()));
        assert!(tables.contains(&"event_streams".to_string()));
        assert!(tables.contains(&"event_subscriptions".to_string()));
        assert!(tables.contains(&"event_delivery_log".to_string()));
        assert!(tables.contains(&"event_replay_log".to_string()));

        // V13
        assert!(tables.contains(&"graph_entities".to_string()));
        assert!(tables.contains(&"graph_relationships".to_string()));
        assert!(tables.contains(&"graph_embeddings".to_string()));
        assert!(tables.contains(&"graph_versions".to_string()));
        assert!(tables.contains(&"entity_aliases".to_string()));
        assert!(tables.contains(&"knowledge_events".to_string()));
        assert!(tables.contains(&"knowledge_projections".to_string()));
        assert!(tables.contains(&"knowledge_cache".to_string()));
        assert!(tables.contains(&"goal_states".to_string()));
        assert!(tables.contains(&"graph_traversals".to_string()));
        assert!(tables.contains(&"graph_communities".to_string()));
        assert!(tables.contains(&"graph_snapshots".to_string()));
        assert!(tables.contains(&"knowledge_provenance".to_string()));
        assert!(tables.contains(&"graph_constraints".to_string()));
        assert!(tables.contains(&"graph_exports".to_string()));

        // V14
        assert!(tables.contains(&"goals".to_string()));
        assert!(tables.contains(&"goal_state_transitions".to_string()));
        assert!(tables.contains(&"plans".to_string()));
        assert!(tables.contains(&"plan_candidates".to_string()));

        // V18
        assert!(tables.contains(&"mission_outcomes".to_string()));
        assert!(tables.contains(&"strategy_history".to_string()));
        assert!(tables.contains(&"learning_profiles".to_string()));

        // V19 — Memory Intelligence Engine
        assert!(tables.contains(&"episodes".to_string()));
        assert!(tables.contains(&"episode_events".to_string()));
        assert!(tables.contains(&"episode_summaries".to_string()));
        assert!(tables.contains(&"semantic_memories".to_string()));
        assert!(tables.contains(&"knowledge_evolution".to_string()));
        assert!(tables.contains(&"memory_clusters".to_string()));
        assert!(tables.contains(&"cluster_memberships".to_string()));
        assert!(tables.contains(&"memory_principles".to_string()));
        assert!(tables.contains(&"decision_history".to_string()));
        assert!(tables.contains(&"experience_reports".to_string()));
        assert!(tables.contains(&"memory_retrieval_log".to_string()));

        // V20 — World Model & Predictive Planning
        assert!(tables.contains(&"world_states".to_string()));
        assert!(tables.contains(&"scenarios".to_string()));
        assert!(tables.contains(&"simulations".to_string()));
        assert!(tables.contains(&"risk_reports".to_string()));
        assert!(tables.contains(&"predictions".to_string()));
        assert!(tables.contains(&"forecast_history".to_string()));
        assert!(tables.contains(&"strategy_rankings".to_string()));
    }
    #[test]
    fn test_migrations_rerun_and_idempotency() {
        let mut conn = Connection::open_in_memory().unwrap();
        // First run
        run(&mut conn).unwrap();

        // Insert some initial test data to verify it is preserved
        conn.execute(
            "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
             VALUES ('proj_1', 'Test', 'Desc', '/root', 'rust', 'infra', 'mature', 100, 100)",
            [],
        ).unwrap();

        // Rerun migrations (second run)
        let result = run(&mut conn);
        assert!(result.is_ok(), "Rerunning migrations failed: {:?}", result);

        // Verify data is still intact
        let project_name: String = conn
            .query_row("SELECT name FROM projects WHERE id = 'proj_1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(project_name, "Test");

        // Rerun migrations again (third run)
        let result_3 = run(&mut conn);
        assert!(
            result_3.is_ok(),
            "Third run of migrations failed: {:?}",
            result_3
        );

        // Verify data is still intact
        let project_name_3: String = conn
            .query_row("SELECT name FROM projects WHERE id = 'proj_1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(project_name_3, "Test");
    }
}
