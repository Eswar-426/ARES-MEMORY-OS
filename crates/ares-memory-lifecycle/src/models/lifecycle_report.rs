use super::freshness_score::FreshnessScore;
use super::lifecycle_state::LifecycleState;
use super::supersession::SupersessionRecord;
use super::trust_score::TrustScore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LifecycleReport {
    pub artifact_id: String,
    pub current_state: LifecycleState,
    pub freshness: FreshnessScore,
    pub trust: TrustScore,
    pub supersession: Option<SupersessionRecord>,
    pub is_archivable: bool,
}
