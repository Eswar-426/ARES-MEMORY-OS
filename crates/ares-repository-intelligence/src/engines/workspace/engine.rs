use ares_core::id::new_id;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentQuestion {
    pub id: String,
    pub question: String,
    pub repository_id: String,
    pub execution_id: String,
    pub replay_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub kind: String, // "Node", "Query", "Search", "Conversation"
    pub value: String,
    pub title: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedNode {
    pub id: String,
    pub node_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationEvent {
    pub id: String,
    pub node_id: String,
    pub timestamp: i64,
}

pub struct WorkspaceEngine {
    conn: Arc<Mutex<Connection>>,
}

impl WorkspaceEngine {
    pub fn new(workspace_dir: PathBuf) -> Result<Self, ares_core::AresError> {
        let db_path = workspace_dir.join("workspace.db");
        let conn = Connection::open(db_path).map_err(|e| {
            ares_core::AresError::Database(format!("Failed to open workspace db: {}", e))
        })?;

        Self::init_schema(&conn).map_err(|e| {
            ares_core::AresError::Database(format!("Failed to init workspace schema: {}", e))
        })?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init_schema(conn: &Connection) -> SqlResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS recent_questions (
                id TEXT PRIMARY KEY,
                question TEXT NOT NULL,
                repository_id TEXT NOT NULL,
                execution_id TEXT NOT NULL,
                replay_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                value TEXT NOT NULL,
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS pins (
                id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS navigation_history (
                id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    // --- Recent Questions ---
    pub async fn add_recent_question(&self, q: RecentQuestion) -> Result<(), ares_core::AresError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO recent_questions (id, question, repository_id, execution_id, replay_id, timestamp) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![q.id, q.question, q.repository_id, q.execution_id, q.replay_id, q.timestamp],
        ).map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_recent_questions(&self) -> Result<Vec<RecentQuestion>, ares_core::AresError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, question, repository_id, execution_id, replay_id, timestamp FROM recent_questions ORDER BY timestamp DESC LIMIT 50")
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(RecentQuestion {
                    id: row.get(0)?,
                    question: row.get(1)?,
                    repository_id: row.get(2)?,
                    execution_id: row.get(3)?,
                    replay_id: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| ares_core::AresError::Database(e.to_string()))?);
        }
        Ok(out)
    }

    // --- Bookmarks ---
    pub async fn bookmark_node(
        &self,
        node_id: &str,
        title: &str,
    ) -> Result<(), ares_core::AresError> {
        self.add_bookmark("Node", node_id, title).await
    }

    pub async fn bookmark_query(
        &self,
        query: &str,
        title: &str,
    ) -> Result<(), ares_core::AresError> {
        self.add_bookmark("Query", query, title).await
    }

    async fn add_bookmark(
        &self,
        kind: &str,
        value: &str,
        title: &str,
    ) -> Result<(), ares_core::AresError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO bookmarks (id, kind, value, title, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![new_id(), kind, value, title, now],
        ).map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_bookmarks(&self) -> Result<Vec<Bookmark>, ares_core::AresError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, value, title, created_at FROM bookmarks ORDER BY created_at DESC",
            )
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Bookmark {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    value: row.get(2)?,
                    title: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| ares_core::AresError::Database(e.to_string()))?);
        }
        Ok(out)
    }

    // --- Pins ---
    pub async fn pin_node(&self, node_id: &str) -> Result<(), ares_core::AresError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        // Upsert logically (delete old then insert or INSERT OR REPLACE if unique constraint existed)
        conn.execute("DELETE FROM pins WHERE node_id = ?1", params![node_id])
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;

        conn.execute(
            "INSERT INTO pins (id, node_id, timestamp) VALUES (?1, ?2, ?3)",
            params![new_id(), node_id, now],
        )
        .map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_pinned_nodes(&self) -> Result<Vec<PinnedNode>, ares_core::AresError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, node_id, timestamp FROM pins ORDER BY timestamp DESC")
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PinnedNode {
                    id: row.get(0)?,
                    node_id: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| ares_core::AresError::Database(e.to_string()))?);
        }
        Ok(out)
    }

    // --- Navigation ---
    pub async fn push_navigation(&self, node_id: &str) -> Result<(), ares_core::AresError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO navigation_history (id, node_id, timestamp) VALUES (?1, ?2, ?3)",
            params![new_id(), node_id, now],
        )
        .map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn navigation_back(
        &self,
        current_timestamp: i64,
    ) -> Result<Option<NavigationEvent>, ares_core::AresError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, node_id, timestamp FROM navigation_history WHERE timestamp < ?1 ORDER BY timestamp DESC LIMIT 1")
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![current_timestamp], |row| {
                Ok(NavigationEvent {
                    id: row.get(0)?,
                    node_id: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;

        if let Some(r) = rows.next() {
            return Ok(Some(
                r.map_err(|e| ares_core::AresError::Database(e.to_string()))?,
            ));
        }
        Ok(None)
    }

    pub async fn navigation_forward(
        &self,
        current_timestamp: i64,
    ) -> Result<Option<NavigationEvent>, ares_core::AresError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, node_id, timestamp FROM navigation_history WHERE timestamp > ?1 ORDER BY timestamp ASC LIMIT 1")
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![current_timestamp], |row| {
                Ok(NavigationEvent {
                    id: row.get(0)?,
                    node_id: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;

        if let Some(r) = rows.next() {
            return Ok(Some(
                r.map_err(|e| ares_core::AresError::Database(e.to_string()))?,
            ));
        }
        Ok(None)
    }
}
