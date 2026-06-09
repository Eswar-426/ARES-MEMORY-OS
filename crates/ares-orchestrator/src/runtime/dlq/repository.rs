use super::models::DeadLetterItem;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

pub struct DlqRepository {
    store: Store,
}

impl DlqRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn insert(&self, item: &DeadLetterItem) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO dead_letter_queue (id, original_queue_id, workflow_id, execution_key, failure_reason, failed_at, attempt_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.id,
                item.original_queue_id,
                item.workflow_id,
                item.execution_key,
                item.failure_reason,
                item.failed_at,
                item.attempt_count
            ],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn list(&self, limit: usize) -> Result<Vec<DeadLetterItem>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, original_queue_id, workflow_id, execution_key, failure_reason, failed_at, attempt_count 
             FROM dead_letter_queue ORDER BY failed_at DESC LIMIT ?1"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(DeadLetterItem {
                id: row.get(0)?,
                original_queue_id: row.get(1)?,
                workflow_id: row.get(2)?,
                execution_key: row.get(3)?,
                failure_reason: row.get(4)?,
                failed_at: row.get(5)?,
                attempt_count: row.get(6)?,
            })
        }).map_err(AresError::db)?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r.map_err(AresError::db)?);
        }
        Ok(items)
    }
}
