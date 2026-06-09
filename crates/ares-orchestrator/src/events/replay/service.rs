use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;

pub struct ReplayEngine {
    store: Store,
}

impl ReplayEngine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn start_replay_job(
        &self,
        target_topic: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<String, AresError> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let conn = self.store.get_conn()?;
        let now = Utc::now().timestamp_millis();

        conn.execute(
            "INSERT INTO event_replay_log (id, replay_job_id, target_topic, start_time, end_time, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'Running', ?6, ?6)",
            params![uuid::Uuid::new_v4().to_string(), job_id, target_topic, start_time, end_time, now],
        ).map_err(AresError::db)?;

        // In a real implementation, this would trigger a background task to fetch events
        // matching the criteria and publish them to a special replay topic,
        // avoiding replay loops by setting a specific `causation_id` or `trace_id`.

        Ok(job_id)
    }
}
