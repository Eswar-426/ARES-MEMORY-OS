use super::models::ProvenanceRecord;
use ares_core::AresError;
use rusqlite::{params, Connection};

pub struct ProvenanceRepository;

impl ProvenanceRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn insert(&self, conn: &Connection, record: &ProvenanceRecord) -> Result<(), AresError> {
        conn.execute(
            "INSERT INTO knowledge_provenance (
                id, entity_id, relationship_id, event_id, source_type, created_by, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.id.to_string(),
                record.entity_id.map(|u| u.to_string()),
                record.relationship_id.map(|u| u.to_string()),
                record.event_id.to_string(),
                record.source_type,
                record.created_by,
                record.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }
}

impl Default for ProvenanceRepository {
    fn default() -> Self {
        Self::new()
    }
}
