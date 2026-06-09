use super::models::{QueueStatus, WorkflowQueueItem};
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

pub struct QueueRepository {
    store: Store,
}

impl QueueRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn enqueue(&self, item: &WorkflowQueueItem) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let status_str = serde_json::to_string(&item.status).unwrap().replace("\"", "");

        // execution_key has a UNIQUE constraint in the DB.
        // If it violates, SQLite will return an error, providing idempotency.
        conn.execute(
            "INSERT INTO workflow_queue (id, workflow_id, priority, status, assigned_worker, retry_count, created_at, started_at, completed_at, execution_key, execution_checksum)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                item.id,
                item.workflow_id,
                item.priority,
                status_str,
                item.assigned_worker,
                item.retry_count,
                item.created_at,
                item.started_at,
                item.completed_at,
                item.execution_key,
                item.execution_checksum
            ],
        ).map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed: workflow_queue.execution_key") {
                AresError::conflict("Duplicate execution key detected")
            } else {
                AresError::db(e)
            }
        })?;
        Ok(())
    }

    pub fn dequeue_unassigned(&self, limit: usize) -> Result<Vec<WorkflowQueueItem>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, workflow_id, priority, status, assigned_worker, retry_count, created_at, started_at, completed_at, execution_key, execution_checksum 
             FROM workflow_queue WHERE status = 'Queued' ORDER BY priority DESC, created_at ASC LIMIT ?1"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let status_str: String = row.get(3)?;
            let status = serde_json::from_str(&format!("\"{}\"", status_str)).unwrap_or(QueueStatus::Queued);

            Ok(WorkflowQueueItem {
                id: row.get(0)?,
                workflow_id: row.get(1)?,
                priority: row.get(2)?,
                status,
                assigned_worker: row.get(4)?,
                retry_count: row.get(5)?,
                created_at: row.get(6)?,
                started_at: row.get(7)?,
                completed_at: row.get(8)?,
                execution_key: row.get(9)?,
                execution_checksum: row.get(10)?,
            })
        }).map_err(AresError::db)?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r.map_err(AresError::db)?);
        }
        Ok(items)
    }

    pub fn update_status(&self, id: &str, status: &QueueStatus, assigned_worker: Option<&str>, started_at: Option<&str>, completed_at: Option<&str>) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let status_str = serde_json::to_string(status).unwrap().replace("\"", "");

        conn.execute(
            "UPDATE workflow_queue SET status = ?1, assigned_worker = ?2, started_at = COALESCE(?3, started_at), completed_at = COALESCE(?4, completed_at) WHERE id = ?5",
            params![status_str, assigned_worker, started_at, completed_at, id],
        ).map_err(AresError::db)?;
        Ok(())
    }
}
