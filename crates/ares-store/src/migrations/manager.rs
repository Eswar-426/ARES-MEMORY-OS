use ares_core::AresError;
use rusqlite::Connection;
use tracing::info;

pub struct MigrationManager;

impl MigrationManager {
    /// Run all database migrations:
    /// 1. Schema migrations (via refinery)
    /// 2. Data migrations (custom logic)
    /// 3. Cleanup migrations
    pub fn run(conn: &mut Connection, current_project_id: &str) -> Result<(), AresError> {
        info!("Starting MigrationManager");

        // 1. Schema Migrations
        Self::run_schema_migrations(conn)?;

        // 2. Data Migrations
        Self::run_data_migrations(conn, current_project_id)?;

        // 3. Cleanup Migrations
        Self::run_cleanup_migrations(conn)?;

        info!("MigrationManager completed successfully");
        Ok(())
    }

    pub fn run_schema_migrations(conn: &mut Connection) -> Result<(), AresError> {
        info!("Running schema migrations...");
        crate::migrations::run(conn)
    }

    fn run_data_migrations(
        conn: &mut Connection,
        current_project_id: &str,
    ) -> Result<(), AresError> {
        info!("Running data migrations...");

        // Ensure migration tracking table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ares_data_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(AresError::db)?;

        // Migration 1: Rename TEST
        let v1_applied: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ares_data_migrations WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if v1_applied == 0 {
            Self::migrate_v1_rename_test(conn, current_project_id)?;
            let now = ares_core::types::event::now_micros();
            conn.execute(
                "INSERT INTO ares_data_migrations (version, name, applied_at) VALUES (1, 'Rename TEST to workspace', ?1)",
                [now],
            ).map_err(AresError::db)?;
        }

        Ok(())
    }

    fn migrate_v1_rename_test(
        conn: &mut Connection,
        current_project_id: &str,
    ) -> Result<(), AresError> {
        info!(
            "Data Migration V1: Renaming legacy 'TEST' project to '{}'",
            current_project_id
        );

        let tx = conn.transaction().map_err(AresError::db)?;

        let _ = tx.execute("UPDATE graph_nodes SET id = ?1, label = ?1, properties = json_set(properties, '$.name', ?1) WHERE node_type = 'project' AND id = 'TEST'", rusqlite::params![current_project_id]);
        let _ = tx.execute(
            "UPDATE graph_edges SET project_id = ?1 WHERE project_id = 'TEST'",
            rusqlite::params![current_project_id],
        );
        let _ = tx.execute(
            "UPDATE graph_edges SET from_node_id = ?1 WHERE from_node_id = 'TEST'",
            rusqlite::params![current_project_id],
        );
        let _ = tx.execute(
            "UPDATE graph_edges SET to_node_id = ?1 WHERE to_node_id = 'TEST'",
            rusqlite::params![current_project_id],
        );
        let _ = tx.execute(
            "UPDATE graph_nodes SET project_id = ?1 WHERE project_id = 'TEST'",
            rusqlite::params![current_project_id],
        );

        tx.commit().map_err(AresError::db)?;

        Ok(())
    }

    fn run_cleanup_migrations(_conn: &mut Connection) -> Result<(), AresError> {
        info!("Running cleanup migrations...");
        Ok(())
    }
}
