use crate::rules::RuleProvider;
use ares_candidates::{
    BootstrapMetadata, Candidate, CandidateConfidence, CandidateStatus, CandidateType,
};
use chrono::Utc;
use uuid::Uuid;

pub struct ArchitectureInferenceEngine {
    pub rules: Vec<Box<dyn RuleProvider>>,
    pub engine_version: String,
}

impl ArchitectureInferenceEngine {
    pub fn new(rules: Vec<Box<dyn RuleProvider>>) -> Self {
        Self {
            rules,
            engine_version: "1.0.0".to_string(),
        }
    }

    pub fn infer(&self, repository_id: &str, commit_hash: &str) -> Vec<Candidate> {
        let mut candidates = Vec::new();

        for provider in &self.rules {
            for rule in provider.load_rules() {
                if rule.target_type == "Architecture" {
                    let candidate = Candidate {
                        id: Uuid::now_v7().to_string(),
                        project_id: repository_id.to_string(),
                        title: format!("Inferred Architecture: {}", rule.inferred_payload),
                        description: format!(
                            "Automatically inferred from pattern: {}",
                            rule.trigger_pattern
                        ),
                        candidate_type: CandidateType::Architecture,
                        decision_category: None,
                        architecture_category: None,
                        traceability_category: None,
                        source_endpoint: None,
                        target_endpoint: None,
                        traceability_strength: None,
                        ownership_domains: vec![],
                        dependent_components: vec![],
                        status: CandidateStatus::Proposed,
                        confidence: CandidateConfidence::from(rule.confidence_score),
                        bootstrap_metadata: Some(BootstrapMetadata {
                            commit_hash: commit_hash.to_string(),
                            repository_id: repository_id.to_string(),
                            rule_id: rule.rule_id.clone(),
                            engine_version: self.engine_version.clone(),
                            generated_at: Utc::now().timestamp(),
                        }),
                        created_at: Utc::now().timestamp(),
                        updated_at: Utc::now().timestamp(),
                    };
                    candidates.push(candidate);
                }
            }
        }
        candidates
    }
}
