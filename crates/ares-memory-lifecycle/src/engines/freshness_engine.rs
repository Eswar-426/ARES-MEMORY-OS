use crate::models::{FreshnessScore, LifecycleState};

pub struct FreshnessEngine {
    pub stale_multiplier: i64,
    pub decay_multiplier: i64,
}

impl FreshnessEngine {
    pub fn new(stale_multiplier: i64, decay_multiplier: i64) -> Self {
        Self {
            stale_multiplier,
            decay_multiplier,
        }
    }

    pub fn evaluate_freshness(
        &self,
        days_since_last_validation: i64,
        baseline_change_frequency_days: i64,
    ) -> FreshnessScore {
        let stale_threshold = baseline_change_frequency_days * self.stale_multiplier;
        let decay_threshold = baseline_change_frequency_days * self.decay_multiplier;

        let is_stale = days_since_last_validation >= stale_threshold;
        let is_decaying = days_since_last_validation >= decay_threshold;

        let score = if is_decaying {
            0.0
        } else if is_stale {
            0.5
        } else {
            1.0
        };

        FreshnessScore {
            score,
            is_stale,
            is_decaying,
            days_since_last_validation,
            baseline_change_frequency_days,
        }
    }

    pub fn determine_state(&self, score: &FreshnessScore) -> LifecycleState {
        if score.is_decaying {
            LifecycleState::Decaying
        } else if score.is_stale {
            LifecycleState::Stale
        } else {
            LifecycleState::Fresh
        }
    }
}
