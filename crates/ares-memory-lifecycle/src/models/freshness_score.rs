use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreshnessScore {
    pub score: f32,
    pub is_stale: bool,
    pub is_decaying: bool,
    pub days_since_last_validation: i64,
    pub baseline_change_frequency_days: i64,
}

impl Default for FreshnessScore {
    fn default() -> Self {
        Self {
            score: 1.0,
            is_stale: false,
            is_decaying: false,
            days_since_last_validation: 0,
            baseline_change_frequency_days: 30, // Default baseline if unknown
        }
    }
}
