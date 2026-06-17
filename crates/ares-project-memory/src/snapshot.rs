//! Snapshot Store — persists, exports, and imports ProjectSnapshots.

use crate::types::ProjectSnapshot;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;
use tracing::info;

pub struct SnapshotStore {
    store: Store,
}

impl SnapshotStore {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Save a snapshot to the database.
    pub fn save(&self, snapshot: &ProjectSnapshot) -> Result<String, AresError> {
        let id = uuid::Uuid::now_v7().to_string();
        let json = serde_json::to_string(snapshot)
            .map_err(|e| AresError::validation(format!("Failed to serialize snapshot: {e}")))?;

        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO project_snapshots (id, project_id, snapshot_json, version, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                snapshot.project_id,
                json,
                snapshot.snapshot_version,
                snapshot.created_at,
            ],
        )
        .map_err(AresError::db)?;

        info!(
            snapshot_id = %id,
            project_id = %snapshot.project_id,
            "Snapshot saved"
        );

        Ok(id)
    }

    /// Load the latest snapshot for a project.
    pub fn get_latest(&self, project_id: &str) -> Result<Option<ProjectSnapshot>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT snapshot_json FROM project_snapshots
                 WHERE project_id = ?1
                 ORDER BY created_at DESC
                 LIMIT 1",
            )
            .map_err(AresError::db)?;

        let result = stmt.query_row(params![project_id], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        });

        match result {
            Ok(json) => {
                let snapshot: ProjectSnapshot = serde_json::from_str(&json)
                    .map_err(|e| AresError::validation(format!("Failed to parse snapshot: {e}")))?;
                Ok(Some(snapshot))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    /// Load a snapshot by ID.
    pub fn get_by_id(&self, snapshot_id: &str) -> Result<Option<ProjectSnapshot>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT snapshot_json FROM project_snapshots WHERE id = ?1")
            .map_err(AresError::db)?;

        let result = stmt.query_row(params![snapshot_id], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        });

        match result {
            Ok(json) => {
                let snapshot: ProjectSnapshot = serde_json::from_str(&json)
                    .map_err(|e| AresError::validation(format!("Failed to parse snapshot: {e}")))?;
                Ok(Some(snapshot))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    /// List snapshots for a project.
    pub fn list(&self, project_id: &str) -> Result<Vec<SnapshotMeta>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, version, created_at FROM project_snapshots
                 WHERE project_id = ?1
                 ORDER BY created_at DESC",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok(SnapshotMeta {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    version: row.get::<_, i64>(2)? as u32,
                    created_at: row.get(3)?,
                })
            })
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    /// Export snapshot as JSON string.
    pub fn export_json(snapshot: &ProjectSnapshot) -> Result<String, AresError> {
        serde_json::to_string_pretty(snapshot)
            .map_err(|e| AresError::validation(format!("Failed to export snapshot: {e}")))
    }

    /// Import snapshot from JSON string.
    pub fn import_json(json: &str) -> Result<ProjectSnapshot, AresError> {
        serde_json::from_str(json)
            .map_err(|e| AresError::validation(format!("Failed to import snapshot: {e}")))
    }
}

/// Lightweight snapshot metadata (without the full JSON payload).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[derive(utoipa::ToSchema)]
pub struct SnapshotMeta {
    pub id: String,
    pub project_id: String,
    pub version: u32,
    pub created_at: i64,
}
