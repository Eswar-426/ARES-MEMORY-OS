use crate::models::LifecycleState;

pub struct RevalidationEngine;

impl Default for RevalidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RevalidationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn attempt_revalidation(&self, state: &LifecycleState, success: bool) -> LifecycleState {
        // Revalidation is only meaningful for Stale or Decaying nodes
        if !matches!(state, LifecycleState::Stale | LifecycleState::Decaying) {
            return state.clone();
        }

        if success {
            LifecycleState::Fresh
        } else {
            LifecycleState::Decaying
        }
    }
}
