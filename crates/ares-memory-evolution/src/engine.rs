use std::sync::Arc;
use ares_core::AresError;
use ares_knowledge_graph::models::DomainEvent;
use crate::models::{EvolutionTimeline, MemoryDiff, MemoryRevision, StateSnapshot, ChangeType};
use crate::store::MemoryEvolutionStore;
use uuid::Uuid;

pub struct MemoryEvolutionEngine {
    store: Arc<MemoryEvolutionStore>,
}

impl MemoryEvolutionEngine {
    pub fn new(store: Arc<MemoryEvolutionStore>) -> Self {
        Self { store }
    }

    /// Idempotent event processing. Records a revision from a DomainEvent.
    pub fn process_event(&self, event: &DomainEvent) -> Result<(), AresError> {
        if self.store.is_event_processed(&event.id)? {
            return Ok(()); // Idempotent skip
        }

        let change_type = match event.event_type {
            ares_knowledge_graph::models::DomainEventType::RequirementCreated => ChangeType::Created,
            ares_knowledge_graph::models::DomainEventType::RequirementUpdated => ChangeType::Updated,
            ares_knowledge_graph::models::DomainEventType::DecisionCreated => ChangeType::Created,
            ares_knowledge_graph::models::DomainEventType::DecisionApproved => ChangeType::Approved,
            _ => ChangeType::Updated, // Fallback, could map entirely based on domain
        };

        // If the event signals a supersession, it should be handled by SupersessionEngine,
        // but for now we log the revision itself.

        let revision = MemoryRevision {
            revision_id: Uuid::now_v7().to_string(),
            entity_id: event.entity_id.clone(),
            entity_type: format!("{:?}", event.event_type).replace("Created", "").replace("Updated", ""),
            change_type,
            changed_at: event.timestamp,
            changed_by: event.payload.get("owner").or_else(|| event.payload.get("approved_by")).and_then(|v| v.as_str()).map(|s| s.to_string()),
            reason: event.payload.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string()),
        };

        self.store.record_revision(&revision)?;

        // Record a dummy diff just for demonstration, ideally derived from before/after in event payload
        let diff = MemoryDiff {
            before: event.payload.get("before_state").cloned().unwrap_or(serde_json::json!({})),
            after: event.payload.get("after_state").cloned().unwrap_or(event.payload.clone()),
        };

        self.store.record_diff(&revision.revision_id, &diff)?;

        self.store.record_event_processed(&event.id, &event.entity_id, event.timestamp)?;

        Ok(())
    }

    pub fn build_timeline(&self, entity_id: &str) -> Result<EvolutionTimeline, AresError> {
        self.store.get_timeline(entity_id)
    }

    pub fn compute_diff(&self, revision_id: &str) -> Result<Option<MemoryDiff>, AresError> {
        self.store.get_diff(revision_id)
    }

    /// Dynamically reconstructs the state snapshot of all entities that were active at a given timestamp.
    pub fn reconstruct_state(&self, timestamp: i64) -> Result<StateSnapshot, AresError> {
        let active_entities = self.store.get_all_entities_changed_before(timestamp)?;
        
        Ok(StateSnapshot {
            timestamp,
            entities: active_entities,
        })
    }

    /// Compares two timestamps and returns the entities that were changed between them.
    pub fn compare_states(&self, t1: i64, t2: i64) -> Result<Vec<String>, AresError> {
        let (start, end) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
        let mut changed_entities = Vec::new();

        let conn = self.store.get_raw_store().get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT entity_id FROM memory_revisions WHERE changed_at > ?1 AND changed_at <= ?2"
        ).map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt.query(rusqlite::params![start, end]).map_err(|e| AresError::Database(e.to_string()))?;
        while let Some(row) = rows.next().map_err(|e| AresError::Database(e.to_string()))? {
            changed_entities.push(row.get(0).map_err(|e| AresError::Database(e.to_string()))?);
        }

        Ok(changed_entities)
    }
}
