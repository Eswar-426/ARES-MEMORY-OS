use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;
use chrono::Utc;

pub struct EventDlqService {
    store: Store,
}

impl EventDlqService {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn add_to_dlq(&self, event_id: &str, group_id: Option<&str>, error_message: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = Utc::now().timestamp_millis();
        
        conn.execute(
            "INSERT INTO event_dlq (id, event_id, group_id, error_message, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'Pending', ?5, ?5)",
            params![uuid::Uuid::new_v4().to_string(), event_id, group_id, error_message, now],
        ).map_err(AresError::db)?;
        
        Ok(())
    }
}
