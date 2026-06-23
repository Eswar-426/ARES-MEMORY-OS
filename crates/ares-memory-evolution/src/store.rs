use crate::models::{ChangeType, EvolutionTimeline, MemoryDiff, MemoryRevision};
use ares_core::AresError;
use ares_store::Store;
use rusqlite::params;
use std::sync::Arc;

pub struct MemoryEvolutionStore {
    raw_store: Arc<Store>,
}

impl MemoryEvolutionStore {
    pub fn new(raw_store: Arc<Store>) -> Self {
        Self { raw_store }
    }

    pub fn get_raw_store(&self) -> Arc<Store> {
        self.raw_store.clone()
    }

    pub fn is_event_processed(&self, event_id: &str) -> Result<bool, AresError> {
        let conn = self.raw_store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT 1 FROM memory_revision_events WHERE event_id = ?1")
            .map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![event_id])
            .map_err(|e| AresError::Database(e.to_string()))?;

        Ok(rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
            .is_some())
    }

    pub fn record_event_processed(
        &self,
        event_id: &str,
        entity_id: &str,
        timestamp: i64,
    ) -> Result<(), AresError> {
        let conn = self.raw_store.get_conn()?;
        conn.execute(
            "INSERT INTO memory_revision_events (event_id, entity_id, processed_at) VALUES (?1, ?2, ?3)",
            params![event_id, entity_id, timestamp],
        ).map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn record_revision(&self, revision: &MemoryRevision) -> Result<(), AresError> {
        let conn = self.raw_store.get_conn()?;

        let change_type_str = serde_json::to_string(&revision.change_type).unwrap_or_default();
        let change_type_str = change_type_str.trim_matches('"');

        conn.execute(
            "INSERT INTO memory_revisions (revision_id, entity_id, entity_type, change_type, changed_at, changed_by, reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                revision.revision_id,
                revision.entity_id,
                revision.entity_type,
                change_type_str,
                revision.changed_at,
                revision.changed_by,
                revision.reason
            ],
        ).map_err(|e| AresError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn record_diff(&self, revision_id: &str, diff: &MemoryDiff) -> Result<(), AresError> {
        let conn = self.raw_store.get_conn()?;

        let before_json = serde_json::to_string(&diff.before).unwrap_or_else(|_| "{}".to_string());
        let after_json = serde_json::to_string(&diff.after).unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "INSERT INTO memory_diffs (revision_id, before_state, after_state)
             VALUES (?1, ?2, ?3)",
            params![revision_id, before_json, after_json],
        )
        .map_err(|e| AresError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn get_revision(&self, revision_id: &str) -> Result<Option<MemoryRevision>, AresError> {
        let conn = self.raw_store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT revision_id, entity_id, entity_type, change_type, changed_at, changed_by, reason 
             FROM memory_revisions WHERE revision_id = ?1"
        ).map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![revision_id])
            .map_err(|e| AresError::Database(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
        {
            let change_type_str: String =
                row.get(3).map_err(|e| AresError::Database(e.to_string()))?;
            let change_type: ChangeType = serde_json::from_str(&format!("\"{}\"", change_type_str))
                .unwrap_or(ChangeType::Updated);

            Ok(Some(MemoryRevision {
                revision_id: row.get(0).map_err(|e| AresError::Database(e.to_string()))?,
                entity_id: row.get(1).map_err(|e| AresError::Database(e.to_string()))?,
                entity_type: row.get(2).map_err(|e| AresError::Database(e.to_string()))?,
                change_type,
                changed_at: row.get(4).map_err(|e| AresError::Database(e.to_string()))?,
                changed_by: row.get(5).unwrap_or_default(),
                reason: row.get(6).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_revisions_for_entity(
        &self,
        entity_id: &str,
    ) -> Result<Vec<MemoryRevision>, AresError> {
        let conn = self.raw_store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT revision_id, entity_id, entity_type, change_type, changed_at, changed_by, reason 
             FROM memory_revisions WHERE entity_id = ?1 ORDER BY changed_at ASC"
        ).map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![entity_id])
            .map_err(|e| AresError::Database(e.to_string()))?;
        let mut revisions = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
        {
            let change_type_str: String =
                row.get(3).map_err(|e| AresError::Database(e.to_string()))?;
            let change_type: ChangeType = serde_json::from_str(&format!("\"{}\"", change_type_str))
                .unwrap_or(ChangeType::Updated);

            revisions.push(MemoryRevision {
                revision_id: row.get(0).map_err(|e| AresError::Database(e.to_string()))?,
                entity_id: row.get(1).map_err(|e| AresError::Database(e.to_string()))?,
                entity_type: row.get(2).map_err(|e| AresError::Database(e.to_string()))?,
                change_type,
                changed_at: row.get(4).map_err(|e| AresError::Database(e.to_string()))?,
                changed_by: row.get(5).unwrap_or_default(),
                reason: row.get(6).unwrap_or_default(),
            });
        }

        Ok(revisions)
    }

    pub fn get_timeline(&self, entity_id: &str) -> Result<EvolutionTimeline, AresError> {
        let revisions = self.get_revisions_for_entity(entity_id)?;
        Ok(EvolutionTimeline {
            entity_id: entity_id.to_string(),
            revisions,
        })
    }

    pub fn get_diff(&self, revision_id: &str) -> Result<Option<MemoryDiff>, AresError> {
        let conn = self.raw_store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT before_state, after_state FROM memory_diffs WHERE revision_id = ?1")
            .map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![revision_id])
            .map_err(|e| AresError::Database(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
        {
            let before_str: String = row.get(0).map_err(|e| AresError::Database(e.to_string()))?;
            let after_str: String = row.get(1).map_err(|e| AresError::Database(e.to_string()))?;

            let before: serde_json::Value =
                serde_json::from_str(&before_str).unwrap_or(serde_json::json!({}));
            let after: serde_json::Value =
                serde_json::from_str(&after_str).unwrap_or(serde_json::json!({}));

            Ok(Some(MemoryDiff { before, after }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_entities_changed_before(
        &self,
        timestamp: i64,
    ) -> Result<Vec<String>, AresError> {
        let conn = self.raw_store.get_conn()?;
        // Get all unique entity IDs that had a change before or exactly at the timestamp
        // Exclude those whose LATEST change before timestamp is 'Archived' if we supported that.
        let mut stmt = conn
            .prepare("SELECT DISTINCT entity_id FROM memory_revisions WHERE changed_at <= ?1")
            .map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![timestamp])
            .map_err(|e| AresError::Database(e.to_string()))?;
        let mut entities = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
        {
            let id: String = row.get(0).map_err(|e| AresError::Database(e.to_string()))?;
            entities.push(id);
        }

        Ok(entities)
    }
}
