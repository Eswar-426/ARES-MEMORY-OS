use serde::{Serialize, Deserialize};
use ares_core::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePromotion {
    pub id: String,
    pub candidate_id: String,
    pub promoted_node_id: NodeId,
    pub promoted_by: String,
    pub promoted_at: i64,
    pub promotion_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateReview {
    pub id: String,
    pub candidate_id: String,
    pub reviewer: String,
    pub comment: String,
    pub status_changed_to: String, // from CandidateStatus
    pub review_date: i64,
}
