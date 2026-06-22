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

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AresError::db(format!("Failed to create parent directory: {}", e))
                })?;
            }
        }

        // Set WAL mode ONCE before spawning concurrent pool connections
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| AresError::db(format!("Failed to open DB for WAL init: {}", e)))?;
        conn.execute_batch("PRAGMA journal_mode = WAL;").ok();
        drop(conn);

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

    /// Computes full graph metrics (Phase 1 Validation).
    pub fn graph_metrics(&self) -> Result<crate::metrics::GraphMetrics, AresError> {
        let conn = self.get_conn()?;

        let total_nodes: usize = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE deleted_at IS NULL",
            (),
            |row| row.get(0),
        ).map_err(|e| AresError::db(e.to_string()))?;

        let total_edges: usize = conn.query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE valid_until IS NULL",
            (),
            |row| row.get(0),
        ).map_err(|e| AresError::db(e.to_string()))?;

        // Calculate orphans efficiently in Rust instead of a nested loop in SQL without index
        let mut all_nodes = std::collections::HashSet::new();
        let mut stmt = conn.prepare("SELECT id FROM graph_nodes WHERE deleted_at IS NULL").unwrap();
        let mut rows = stmt.query(()).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let id: String = row.get(0).unwrap();
            all_nodes.insert(id);
        }

        let mut connected_nodes = std::collections::HashSet::new();
        let mut stmt = conn.prepare("SELECT from_node_id, to_node_id FROM graph_edges WHERE valid_until IS NULL").unwrap();
        let mut rows = stmt.query(()).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let from: String = row.get(0).unwrap();
            let to: String = row.get(1).unwrap();
            connected_nodes.insert(from);
            connected_nodes.insert(to);
        }

        let orphan_nodes = all_nodes.difference(&connected_nodes).count();

        let mut node_type_counts = std::collections::HashMap::new();
        let mut stmt = conn.prepare("SELECT node_type, COUNT(*) FROM graph_nodes WHERE deleted_at IS NULL GROUP BY node_type").map_err(|e| AresError::db(e.to_string()))?;
        let mut rows = stmt.query(()).map_err(|e| AresError::db(e.to_string()))?;
        while let Some(row) = rows.next().map_err(|e| AresError::db(e.to_string()))? {
            let t: String = row.get(0).unwrap_or_default();
            let c: usize = row.get(1).unwrap_or_default();
            node_type_counts.insert(t, c);
        }

        let mut relationship_type_counts = std::collections::HashMap::new();
        let mut stmt = conn.prepare("SELECT edge_type, COUNT(*) FROM graph_edges WHERE valid_until IS NULL GROUP BY edge_type").map_err(|e| AresError::db(e.to_string()))?;
        let mut rows = stmt.query(()).map_err(|e| AresError::db(e.to_string()))?;
        while let Some(row) = rows.next().map_err(|e| AresError::db(e.to_string()))? {
            let t: String = row.get(0).unwrap_or_default();
            let c: usize = row.get(1).unwrap_or_default();
            relationship_type_counts.insert(t, c);
        }

        // Compute largest connected component in memory using BFS/Union-Find
        let mut parent: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut size: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        let mut stmt = conn.prepare("SELECT id FROM graph_nodes WHERE deleted_at IS NULL").map_err(|e| AresError::db(e.to_string()))?;
        let mut rows = stmt.query(()).map_err(|e| AresError::db(e.to_string()))?;
        while let Some(row) = rows.next().map_err(|e| AresError::db(e.to_string()))? {
            let id: String = row.get(0).unwrap();
            parent.insert(id.clone(), id.clone());
            size.insert(id, 1);
        }

        fn find(parent: &mut std::collections::HashMap<String, String>, i: String) -> String {
            let mut curr = i;
            while parent[&curr] != curr {
                let p = parent[&curr].clone();
                let gp = parent[&p].clone();
                parent.insert(curr.clone(), gp);
                curr = p;
            }
            curr
        }

        let mut stmt = conn.prepare("SELECT from_node_id, to_node_id FROM graph_edges WHERE valid_until IS NULL").map_err(|e| AresError::db(e.to_string()))?;
        let mut rows = stmt.query(()).map_err(|e| AresError::db(e.to_string()))?;
        while let Some(row) = rows.next().map_err(|e| AresError::db(e.to_string()))? {
            let s: String = row.get(0).unwrap();
            let t: String = row.get(1).unwrap();
            if parent.contains_key(&s) && parent.contains_key(&t) {
                let root_s = find(&mut parent, s);
                let root_t = find(&mut parent, t);
                if root_s != root_t {
                    let size_s = size[&root_s];
                    let size_t = size[&root_t];
                    if size_s < size_t {
                        parent.insert(root_s.clone(), root_t.clone());
                        size.insert(root_t, size_s + size_t);
                    } else {
                        parent.insert(root_t.clone(), root_s.clone());
                        size.insert(root_s, size_s + size_t);
                    }
                }
            }
        }

        let largest_connected_component = size.values().copied().max().unwrap_or(0);
        
        let average_degree = if total_nodes > 0 {
            (total_edges as f64 * 2.0) / (total_nodes as f64)
        } else {
            0.0
        };

        let graph_density = if total_nodes > 1 {
            (total_edges as f64 * 2.0) / ((total_nodes as f64) * (total_nodes as f64 - 1.0))
        } else {
            0.0
        };

        Ok(crate::metrics::GraphMetrics {
            total_nodes,
            total_edges,
            node_type_counts,
            relationship_type_counts,
            orphan_nodes,
            largest_connected_component,
            average_degree,
            graph_density,
        })
    }

    pub fn call_graph_metrics(&self) -> Result<crate::metrics::CallGraphMetrics, AresError> {
        let conn = self.get_conn()?;

        let call_edges: usize = conn.query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE edge_type = 'calls' AND valid_until IS NULL",
            (),
            |row| row.get(0),
        ).unwrap_or(0);

        let dependency_edges: usize = conn.query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE edge_type IN ('depends_on', 'imports', 'uses_module', 'uses_trait', 'contained_in') AND valid_until IS NULL",
            (),
            |row| row.get(0),
        ).unwrap_or(0);

        let implementation_edges: usize = conn.query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE edge_type = 'implements' AND valid_until IS NULL",
            (),
            |row| row.get(0),
        ).unwrap_or(0);

        let resolved_symbols: usize = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE properties NOT LIKE '%\"unresolved\":true%' AND deleted_at IS NULL",
            (),
            |row| row.get(0),
        ).unwrap_or(0);

        let unresolved_symbols: usize = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE properties LIKE '%\"unresolved\":true%' AND deleted_at IS NULL",
            (),
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(crate::metrics::CallGraphMetrics {
            call_edges,
            dependency_edges,
            implementation_edges,
            resolved_symbols,
            unresolved_symbols,
        })
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
        PRAGMA foreign_keys = ON;
        PRAGMA synchronous = NORMAL;
        PRAGMA temp_store = MEMORY;
        PRAGMA mmap_size = 268435456;
        PRAGMA cache_size = -8000;
        PRAGMA busy_timeout = 5000;
        ",
    )
}

// Available to other crates for testing
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
    fn test_store_open_creates_parent_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        // Path with a non-existent parent directory
        let db_path = dir.path().join("nested").join("folder").join("test.db");

        let store = Store::open(&db_path);
        assert!(
            store.is_ok(),
            "Store::open should succeed and create parent directories"
        );
        assert!(db_path.exists(), "Database file should exist");
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
