use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;

pub struct AggregateService {
    store: Store,
}

impl AggregateService {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn save_state(
        &self,
        id: &str,
        aggregate_type: &str,
        version: u32,
        state: &str,
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = Utc::now().timestamp_millis();

        conn.execute(
            "INSERT INTO event_aggregates (id, aggregate_type, version, state, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(id) DO UPDATE SET version=excluded.version, state=excluded.state, updated_at=excluded.updated_at",
            params![id, aggregate_type, version, state, now],
        ).map_err(AresError::db)?;

        Ok(())
    }
}
