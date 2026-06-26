use ares_candidates::{Candidate, CandidateStatus};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateGovernancePolicy {
    pub expiration_days: i64,
    pub revalidation_days: i64,
    pub auto_expire_low_confidence: bool,
    pub bulk_promotion_threshold: f64,
}

impl Default for CandidateGovernancePolicy {
    fn default() -> Self {
        Self {
            expiration_days: 90,
            revalidation_days: 30,
            auto_expire_low_confidence: true,
            bulk_promotion_threshold: 0.95,
        }
    }
}

pub struct CandidateGovernanceEngine {
    policy: CandidateGovernancePolicy,
}

impl CandidateGovernanceEngine {
    pub fn new(policy: CandidateGovernancePolicy) -> Self {
        Self { policy }
    }

    /// Evaluates if a candidate should be expired based on its age and the governance policy.
    pub fn evaluate_expiration(&self, candidate: &mut Candidate) -> bool {
        if candidate.status == CandidateStatus::Approved
            || candidate.status == CandidateStatus::Rejected
            || candidate.status == CandidateStatus::Superseded
        {
            return false;
        }

        let now = Utc::now().timestamp();
        let age_seconds = now - candidate.created_at;
        let age_days = age_seconds / (24 * 3600);

        if age_days >= self.policy.expiration_days {
            candidate.status = CandidateStatus::Rejected;
            // Optionally: add a review/log indicating expiration
            return true;
        }

        if self.policy.auto_expire_low_confidence && candidate.confidence.evidence_count == 0 {
            candidate.status = CandidateStatus::Rejected;
            return true;
        }

        false
    }

    /// Evaluates if a candidate requires revalidation based on changed factors.
    pub fn requires_revalidation(
        &self,
        candidate: &Candidate,
        current_commit_hash: &str,
        _current_rule_version: &str,
        current_engine_version: &str,
        evidence_exists: bool,
    ) -> bool {
        // If it's already resolved, no revalidation needed
        if candidate.status != CandidateStatus::Proposed
            && candidate.status != CandidateStatus::UnderReview
        {
            return false;
        }

        if !evidence_exists {
            return true;
        }

        if let Some(meta) = &candidate.bootstrap_metadata {
            if meta.commit_hash != current_commit_hash {
                return true;
            }
            if meta.engine_version != current_engine_version {
                return true;
            }
        }

        false
    }

    /// Returns true if the candidate meets the threshold for bulk promotion.
    pub fn meets_bulk_promotion_threshold(&self, candidate: &Candidate, score: f64) -> bool {
        if candidate.status != CandidateStatus::Proposed
            && candidate.status != CandidateStatus::UnderReview
        {
            return false;
        }

        score >= self.policy.bulk_promotion_threshold
    }
}
