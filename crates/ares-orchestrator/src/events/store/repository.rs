use crate::events::envelope::EventEnvelope;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

pub struct EventStoreRepository {
    store: Store,
}

impl EventStoreRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn insert(&self, event: &EventEnvelope) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        
        let payload_str = serde_json::to_string(&event.payload).unwrap_or_default();
        let metadata_str = serde_json::to_string(&event.metadata).unwrap_or_default();
        
        conn.execute(
            "INSERT INTO event_store (
                id, topic, event_type, source, schema_version, event_version, 
                correlation_id, causation_id, trace_id, partition_key, 
                payload, metadata, timestamp, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                event.id,
                event.topic,
                event.event_type,
                event.source,
                event.schema_version,
                event.event_version,
                event.correlation_id,
                event.causation_id,
                event.trace_id,
                event.partition_key,
                payload_str,
                metadata_str,
                event.timestamp.timestamp_millis(),
                chrono::Utc::now().timestamp_millis()
            ],
        ).map_err(AresError::db)?;
        
        Ok(())
    }
}
