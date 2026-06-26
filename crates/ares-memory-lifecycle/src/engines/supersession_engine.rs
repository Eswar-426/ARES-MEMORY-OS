use crate::models::{LifecycleState, SupersessionRecord};

pub struct SupersessionEngine;

impl Default for SupersessionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SupersessionEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_supersession(
        &self,
        records: &[SupersessionRecord],
    ) -> Option<SupersessionRecord> {
        // For deterministic behavior, pick the record with the highest confidence
        let mut best_record: Option<SupersessionRecord> = None;
        let mut max_confidence = -1.0;

        for record in records {
            if record.confidence > max_confidence {
                max_confidence = record.confidence;
                best_record = Some(record.clone());
            }
        }

        best_record
    }

    pub fn apply_supersession(
        &self,
        current_state: &LifecycleState,
        best_record: &Option<SupersessionRecord>,
    ) -> LifecycleState {
        if best_record.is_some() {
            LifecycleState::Superseded
        } else {
            current_state.clone()
        }
    }
}
