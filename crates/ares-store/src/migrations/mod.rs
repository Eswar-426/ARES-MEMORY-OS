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
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"memories".to_string()));
        assert!(tables.contains(&"decisions".to_string()));
        assert!(tables.contains(&"graph_nodes".to_string()));
        assert!(tables.contains(&"graph_edges".to_string()));
        assert!(tables.contains(&"events".to_string()));
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
        let project_name: String = conn.query_row(
            "SELECT name FROM projects WHERE id = 'proj_1'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(project_name, "Test");

        // Rerun migrations again (third run)
        let result_3 = run(&mut conn);
        assert!(result_3.is_ok(), "Third run of migrations failed: {:?}", result_3);

        // Verify data is still intact
        let project_name_3: String = conn.query_row(
            "SELECT name FROM projects WHERE id = 'proj_1'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(project_name_3, "Test");
    }
}
