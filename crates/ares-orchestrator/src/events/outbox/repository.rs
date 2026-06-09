use super::models::OutboxEvent;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;

pub struct OutboxRepository {
    store: Store,
}

impl OutboxRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn insert(&self, event: &OutboxEvent) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO outbox_events (id, topic, payload, created_at, published_at, status, retry_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![event.id, event.topic, event.payload, event.created_at, event.published_at, event.status, event.retry_count],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn fetch_pending(&self, limit: usize) -> Result<Vec<OutboxEvent>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, topic, payload, created_at, published_at, status, retry_count FROM outbox_events WHERE status = 'Pending' ORDER BY created_at ASC LIMIT ?1"
        ).map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(OutboxEvent {
                    id: row.get(0)?,
                    topic: row.get(1)?,
                    payload: row.get(2)?,
                    created_at: row.get(3)?,
                    published_at: row.get(4)?,
                    status: row.get(5)?,
                    retry_count: row.get(6)?,
                })
            })
            .map_err(AresError::db)?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r.map_err(AresError::db)?);
        }
        Ok(items)
    }

    pub fn mark_published(&self, id: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE outbox_events SET status = 'Published', published_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    pub fn increment_retry(&self, id: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE outbox_events SET retry_count = retry_count + 1 WHERE id = ?1",
            params![id],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    pub fn mark_failed(&self, id: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE outbox_events SET status = 'Failed' WHERE id = ?1",
            params![id],
        )
        .map_err(AresError::db)?;
        Ok(())
    }
}
