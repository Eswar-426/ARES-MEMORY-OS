use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;

pub struct SnapshotRepository {
    store: Store,
}

impl SnapshotRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn insert_snapshot(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        version: u32,
        snapshot_data: &str,
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = Utc::now().timestamp_millis();

        conn.execute(
            "INSERT INTO event_snapshots (id, aggregate_id, aggregate_type, version, snapshot_data, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![uuid::Uuid::new_v4().to_string(), aggregate_id, aggregate_type, version, snapshot_data, now],
        ).map_err(AresError::db)?;

        Ok(())
    }
}
