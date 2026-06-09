use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;

pub struct ConsumerGroupService {
    store: Store,
}

impl ConsumerGroupService {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn commit_offset(
        &self,
        group_id: &str,
        partition_key: &str,
        event_id: &str,
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = Utc::now().timestamp_millis();

        conn.execute(
            "INSERT INTO event_group_offsets (group_id, partition_key, last_processed_event_id, last_processed_timestamp, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(group_id, partition_key) DO UPDATE SET last_processed_event_id=excluded.last_processed_event_id, last_processed_timestamp=excluded.last_processed_timestamp, updated_at=excluded.updated_at",
            params![group_id, partition_key, event_id, now, now],
        ).map_err(AresError::db)?;

        Ok(())
    }
}
