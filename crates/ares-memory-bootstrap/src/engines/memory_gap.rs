use ares_candidates::{
    BootstrapMetadata, Candidate, CandidateConfidence, CandidateStatus, CandidateType,
};
use ares_gap_engine::models::{Gap, GapType};
use chrono::Utc;
use uuid::Uuid;

pub enum BootstrapResult {
    Inferred(Box<Candidate>),
    NeedsHumanReview(String),
}

pub struct MemoryGapBootstrapEngine {
    pub engine_version: String,
}

impl Default for MemoryGapBootstrapEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryGapBootstrapEngine {
    pub fn new() -> Self {
        Self {
            engine_version: "1.0.0".to_string(),
        }
    }

    pub fn attempt_bootstrap(
        &self,
        repository_id: &str,
        commit_hash: &str,
        gap: &Gap,
    ) -> BootstrapResult {
        match gap.gap_type {
            GapType::OrphanCode => {
                // If there's orphan code, we might infer a requirement
                let candidate = Candidate {
                    id: Uuid::now_v7().to_string(),
                    project_id: repository_id.to_string(),
                    title: format!("Inferred Requirement for Orphan Code: {}", gap.source_id),
                    description: format!(
                        "Automatically inferred requirement due to Gap {}",
                        gap.id
                    ),
                    candidate_type: CandidateType::Requirement,
                    decision_category: None,
                    architecture_category: None,
                    traceability_category: None,
                    source_endpoint: None,
                    target_endpoint: None,
                    traceability_strength: None,
                    ownership_domains: vec![],
                    dependent_components: vec![],
                    status: CandidateStatus::Proposed,
                    confidence: CandidateConfidence::from(0.6),
                    bootstrap_metadata: Some(BootstrapMetadata {
                        commit_hash: commit_hash.to_string(),
                        repository_id: repository_id.to_string(),
                        rule_id: "gap_orphan_bootstrap".to_string(),
                        engine_version: self.engine_version.clone(),
                        generated_at: Utc::now().timestamp(),
                    }),
                    created_at: Utc::now().timestamp(),
                    updated_at: Utc::now().timestamp(),
                };
                BootstrapResult::Inferred(Box::new(candidate))
            }
            GapType::MissingDecision => {
                // We can't always infer a decision just because it's missing, might need a human
                BootstrapResult::NeedsHumanReview(
                    "Decision cannot be inferred with high confidence from this gap.".to_string(),
                )
            }
            _ => BootstrapResult::NeedsHumanReview(
                "No bootstrap rule available for this gap type.".to_string(),
            ),
        }
    }
}
