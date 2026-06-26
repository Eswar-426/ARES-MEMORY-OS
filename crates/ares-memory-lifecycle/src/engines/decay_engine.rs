use crate::models::{FreshnessScore, LifecycleState};

pub struct DecayEngine;

impl Default for DecayEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DecayEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_decay(
        &self,
        current_state: &LifecycleState,
        freshness: &FreshnessScore,
        is_orphaned: bool,
        is_unused: bool,
    ) -> bool {
        // If it's already decaying, it stays decaying
        if matches!(current_state, LifecycleState::Decaying) {
            return true;
        }

        // If it is orphaned or unused, it is decaying
        if is_orphaned || is_unused {
            return true;
        }

        // If the freshness score says it's decaying
        if freshness.is_decaying {
            return true;
        }

        false
    }
}
