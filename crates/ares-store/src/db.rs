use ares_core::AresError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use tracing::{debug, info};

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConn = r2d2::PooledConnection<SqliteConnectionManager>;

/// The central store — wraps the connection pool and provides helpers.
///
/// One `Store` per project database file. All repositories share the pool.
#[derive(Clone)]
pub struct Store {
    pool: DbPool,
}

impl Store {
    /// Open (or create) the SQLite database at the given path.
    /// Runs all pending migrations before returning.
    pub fn open(path: &Path) -> Result<Self, AresError> {
        info!(db_path = %path.display(), "Opening ARES database");

        let manager =
            SqliteConnectionManager::file(path).with_init(|conn| configure_connection(conn));

        let pool = Pool::builder()
            .max_size(8)
            .connection_timeout(std::time::Duration::from_secs(5))
            .build(manager)
            .map_err(|e| AresError::migration(format!("Failed to build connection pool: {e}")))?;

        let store = Self { pool };
        store.run_migrations()?;

        Ok(store)
    }

    /// Run all pending schema migrations.
    pub fn run_migrations(&self) -> Result<(), AresError> {
        debug!("Running schema migrations");
        let mut conn = self.get_conn()?;
        crate::migrations::run(&mut conn)
    }

    /// Acquire a connection from the pool.
    pub fn get_conn(&self) -> Result<DbConn, AresError> {
        self.pool
            .get()
            .map_err(|e| AresError::db(format!("Failed to get connection: {e}")))
    }

    /// Execute a closure within a transaction.
    /// Rolls back automatically if the closure returns Err.
    pub fn with_transaction<F, T>(&self, f: F) -> Result<T, AresError>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> Result<T, AresError>,
    {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction().map_err(AresError::db)?;
        match f(&tx) {
            Ok(result) => {
                tx.commit().map_err(AresError::db)?;
                Ok(result)
            }
            Err(e) => {
                // tx rolls back on drop — no explicit rollback needed
                Err(e)
            }
        }
    }

    /// Run a read-only integrity check.
    /// Returns Ok(()) if the database is healthy.
    pub fn integrity_check(&self) -> Result<(), AresError> {
        let conn = self.get_conn()?;
        let result: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(AresError::db)?;
        if result != "ok" {
            return Err(AresError::conflict(format!(
                "Database integrity check failed: {result}"
            )));
        }
        Ok(())
    }
}

/// Configure a new SQLite connection with optimal settings for ARES.
fn configure_connection(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        PRAGMA synchronous = NORMAL;
        PRAGMA temp_store = MEMORY;
        PRAGMA mmap_size = 268435456;
        PRAGMA cache_size = -8000;
        PRAGMA busy_timeout = 5000;
        ",
    )
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use tempfile::TempDir;

    /// Create an in-memory test store with migrations applied.
    /// Returns both the store and the temp dir (keep temp dir alive!).
    pub fn test_store() -> (Store, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = dir.path().join("test.db");
        let store = Store::open(&db_path).expect("Failed to open test store");
        (store, dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_initialization_settings() {
        let (store, _dir) = test_helpers::test_store();
        let conn = store.get_conn().unwrap();

        // Verify journal mode is WAL
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode, "wal");

        // Verify foreign keys are enabled
        let foreign_keys: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(foreign_keys, 1);

        // Verify synchronous mode is NORMAL (1)
        let synchronous: i32 = conn
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .unwrap();
        assert_eq!(synchronous, 1);

        // Verify busy timeout is configured to 5000ms
        let busy_timeout: i32 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .unwrap();
        assert_eq!(busy_timeout, 5000);
    }

    #[test]
    fn test_store_open_invalid_path() {
        // Attempting to open a path where the parent directory does not exist
        let invalid_path = std::path::Path::new("/nonexistent_directory_xyz/db.sqlite");
        let result = Store::open(invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_integrity_check() {
        let (store, _dir) = test_helpers::test_store();
        assert!(store.integrity_check().is_ok());
    }

    #[test]
    fn test_transaction_commit() {
        let (store, _dir) = test_helpers::test_store();

        // Successful transaction commits
        store.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
                 VALUES ('p1', 'P1', '', '/p1', 'rust', '', 'mature', 0, 0)",
                [],
            ).map_err(AresError::db)?;
            Ok(())
        }).unwrap();

        // Verify project was inserted
        let conn = store.get_conn().unwrap();
        let exists: i32 = conn
            .query_row("SELECT COUNT(*) FROM projects WHERE id='p1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(exists, 1);
    }

    #[test]
    fn test_transaction_rollback() {
        let (store, _dir) = test_helpers::test_store();

        // Failing transaction rolls back
        let result: Result<(), AresError> = store.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
                 VALUES ('p2', 'P2', '', '/p2', 'rust', '', 'mature', 0, 0)",
                [],
            ).map_err(AresError::db)?;
            // Force error to rollback
            Err(AresError::validation("rollback this please"))
        });

        assert!(result.is_err());

        // Verify project was NOT inserted
        let conn = store.get_conn().unwrap();
        let exists: i32 = conn
            .query_row("SELECT COUNT(*) FROM projects WHERE id='p2'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(exists, 0);
    }
}
