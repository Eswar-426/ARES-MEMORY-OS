use crate::models::LifecycleState;

pub struct ArchivalEngine {
    pub superseded_after_days: i64,
}

impl ArchivalEngine {
    pub fn new(superseded_after_days: i64) -> Self {
        Self {
            superseded_after_days,
        }
    }

    pub fn determine_archival(
        &self,
        current_state: &LifecycleState,
        days_since_superseded: Option<i64>,
        is_unused: bool,
    ) -> LifecycleState {
        if is_unused {
            return LifecycleState::Archived;
        }

        if matches!(current_state, LifecycleState::Superseded) {
            if let Some(days) = days_since_superseded {
                if days >= self.superseded_after_days {
                    return LifecycleState::Archived;
                }
            }
        }

        current_state.clone()
    }
}
