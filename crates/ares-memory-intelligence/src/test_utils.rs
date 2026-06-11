/// Test utilities for ares-memory-intelligence.
/// These exist because ares-store's `test_helpers` module is `#[cfg(test)]`
/// and therefore unavailable to dependent crates at test time.
use ares_store::db::Store;
use tempfile::TempDir;

/// Create a test store backed by a temp directory with all migrations applied.
/// Foreign key enforcement is disabled so that unit tests can insert into
/// child tables (experiences, decisions, semantic memories) without needing
/// to populate every parent table first.
/// **Keep the returned `TempDir` alive** for the duration of the test.
pub fn test_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let store = Store::open(&db_path).expect("Failed to open test store");
    // Disable FK constraints for isolated unit tests.
    {
        let conn = store.get_conn().expect("Failed to get connection");
        conn.execute_batch("PRAGMA foreign_keys = OFF;")
            .expect("Failed to disable FK constraints");
    }
    (store, dir)
}
