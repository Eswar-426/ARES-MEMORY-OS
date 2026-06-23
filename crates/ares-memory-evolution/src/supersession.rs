use ares_core::AresError;
use ares_store::Store;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySupersession {
    pub supersession_id: String,
    pub superseded_entity_id: String,
    pub superseding_entity_id: String,
    pub entity_type: String,
    pub superseded_at: i64,
    pub reason: Option<String>,
}

pub struct SupersessionEngine {
    raw_store: Arc<Store>,
}

impl SupersessionEngine {
    pub fn new(raw_store: Arc<Store>) -> Self {
        Self { raw_store }
    }

    pub fn record_supersession(&self, supersession: &EntitySupersession) -> Result<(), AresError> {
        let conn = self.raw_store.get_conn()?;
        conn.execute(
            "INSERT INTO entity_supersession 
             (supersession_id, superseded_entity_id, superseding_entity_id, entity_type, superseded_at, reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                supersession.supersession_id,
                supersession.superseded_entity_id,
                supersession.superseding_entity_id,
                supersession.entity_type,
                supersession.superseded_at,
                supersession.reason
            ],
        ).map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn what_replaced_this(
        &self,
        entity_id: &str,
    ) -> Result<Vec<EntitySupersession>, AresError> {
        let conn = self.raw_store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT supersession_id, superseded_entity_id, superseding_entity_id, entity_type, superseded_at, reason
             FROM entity_supersession WHERE superseded_entity_id = ?1 ORDER BY superseded_at ASC"
        ).map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![entity_id])
            .map_err(|e| AresError::Database(e.to_string()))?;
        let mut results = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
        {
            results.push(EntitySupersession {
                supersession_id: row.get(0).map_err(|e| AresError::Database(e.to_string()))?,
                superseded_entity_id: row.get(1).map_err(|e| AresError::Database(e.to_string()))?,
                superseding_entity_id: row
                    .get(2)
                    .map_err(|e| AresError::Database(e.to_string()))?,
                entity_type: row.get(3).map_err(|e| AresError::Database(e.to_string()))?,
                superseded_at: row.get(4).map_err(|e| AresError::Database(e.to_string()))?,
                reason: row.get(5).unwrap_or_default(),
            });
        }

        Ok(results)
    }

    pub fn what_was_replaced_by_this(
        &self,
        entity_id: &str,
    ) -> Result<Vec<EntitySupersession>, AresError> {
        let conn = self.raw_store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT supersession_id, superseded_entity_id, superseding_entity_id, entity_type, superseded_at, reason
             FROM entity_supersession WHERE superseding_entity_id = ?1 ORDER BY superseded_at ASC"
        ).map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![entity_id])
            .map_err(|e| AresError::Database(e.to_string()))?;
        let mut results = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
        {
            results.push(EntitySupersession {
                supersession_id: row.get(0).map_err(|e| AresError::Database(e.to_string()))?,
                superseded_entity_id: row.get(1).map_err(|e| AresError::Database(e.to_string()))?,
                superseding_entity_id: row
                    .get(2)
                    .map_err(|e| AresError::Database(e.to_string()))?,
                entity_type: row.get(3).map_err(|e| AresError::Database(e.to_string()))?,
                superseded_at: row.get(4).map_err(|e| AresError::Database(e.to_string()))?,
                reason: row.get(5).unwrap_or_default(),
            });
        }

        Ok(results)
    }
}
