use crate::engine::MemoryEvolutionEngine;
use crate::models::{EvolutionTimeline, MemoryRevision, StateSnapshot};
use crate::supersession::{EntitySupersession, SupersessionEngine};
use ares_core::AresError;
use std::sync::Arc;

pub struct TemporalQueries {
    engine: Arc<MemoryEvolutionEngine>,
    supersession: Arc<SupersessionEngine>,
}

impl TemporalQueries {
    pub fn new(engine: Arc<MemoryEvolutionEngine>, supersession: Arc<SupersessionEngine>) -> Self {
        Self {
            engine,
            supersession,
        }
    }

    pub fn how_has_this_evolved(&self, entity_id: &str) -> Result<EvolutionTimeline, AresError> {
        self.engine.build_timeline(entity_id)
    }

    pub fn show_state_at_time(&self, timestamp: i64) -> Result<StateSnapshot, AresError> {
        self.engine.reconstruct_state(timestamp)
    }

    pub fn show_changes_between(&self, t1: i64, t2: i64) -> Result<Vec<String>, AresError> {
        self.engine.compare_states(t1, t2)
    }

    pub fn why_was_this_changed(&self, entity_id: &str) -> Result<Vec<MemoryRevision>, AresError> {
        let timeline = self.engine.build_timeline(entity_id)?;
        Ok(timeline
            .revisions
            .into_iter()
            .filter(|r| r.reason.is_some())
            .collect())
    }

    pub fn what_replaced_this(
        &self,
        entity_id: &str,
    ) -> Result<Vec<EntitySupersession>, AresError> {
        self.supersession.what_replaced_this(entity_id)
    }

    pub fn what_was_replaced(&self, entity_id: &str) -> Result<Vec<EntitySupersession>, AresError> {
        self.supersession.what_was_replaced_by_this(entity_id)
    }

    pub fn who_changed_this(&self, entity_id: &str) -> Result<Vec<String>, AresError> {
        let timeline = self.engine.build_timeline(entity_id)?;
        let mut authors = Vec::new();
        for rev in timeline.revisions {
            if let Some(author) = rev.changed_by {
                if !authors.contains(&author) {
                    authors.push(author);
                }
            }
        }
        Ok(authors)
    }

    pub fn what_was_active_during(&self, timestamp: i64) -> Result<Vec<String>, AresError> {
        // Alias for show_state_at_time
        let snapshot = self.engine.reconstruct_state(timestamp)?;
        Ok(snapshot.entities)
    }
}
