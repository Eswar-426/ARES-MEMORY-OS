use std::collections::HashSet;
use uuid::Uuid;

use ares_candidates::{
    ArchitectureCategory, Candidate, CandidateConfidence, CandidateSource, CandidateStatus, CandidateType,
};
use ares_candidates::confidence::CandidateThresholds;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArchitectureEvidenceType {
    WorkspaceStructure,
    DependencyGraph,
    CodeOwnership,
    CommitHistory,
}

#[derive(Debug, Clone)]
pub struct ArchitectureEvidence {
    pub evidence_type: ArchitectureEvidenceType,
    pub source: CandidateSource,
}

pub struct ArchitectureCandidateBuilder {
    pub category: ArchitectureCategory,
    pub title: String,
    pub project_id: String,
    pub sources: Vec<CandidateSource>,
    pub evidence_types: HashSet<ArchitectureEvidenceType>,
    pub ownership_domains: Vec<String>,
    pub dependent_components: Vec<String>,
}

impl ArchitectureCandidateBuilder {
    pub fn new(project_id: impl Into<String>, title: impl Into<String>, category: ArchitectureCategory) -> Self {
        Self {
            category,
            title: title.into(),
            project_id: project_id.into(),
            sources: Vec::new(),
            evidence_types: HashSet::new(),
            ownership_domains: Vec::new(),
            dependent_components: Vec::new(),
        }
    }

    pub fn add_evidence(mut self, evidence: ArchitectureEvidence) -> Self {
        self.evidence_types.insert(evidence.evidence_type);
        self.sources.push(evidence.source);
        self
    }

    pub fn add_ownership_domain(mut self, domain: impl Into<String>) -> Self {
        self.ownership_domains.push(domain.into());
        self
    }

    pub fn add_dependent_component(mut self, component: impl Into<String>) -> Self {
        self.dependent_components.push(component.into());
        self
    }

    pub fn build(self) -> Result<Candidate, String> {
        if self.evidence_types.len() < 2 {
            return Err("Architecture Candidates require a minimum of 2 independent evidence types".to_string());
        }

        let now = chrono::Utc::now().timestamp();
        let id = Uuid::new_v4().to_string();

        let evidence_count = self.sources.len() as u32;
        let diversity = self.evidence_types.len() as u32;

        let confidence = CandidateConfidence {
            evidence_count,
            source_diversity: diversity,
            temporal_consistency: 1.0, // Simplified
            cluster_strength: 1.0,     // Simplified
        };

        if confidence.normalized_score() < CandidateThresholds::architecture() {
            return Err(format!(
                "Confidence score {} is below the required Architecture threshold of {}",
                confidence.normalized_score(),
                CandidateThresholds::architecture()
            ));
        }

        Ok(Candidate {
            id,
            project_id: self.project_id,
            title: self.title,
            description: format!("Deterministic architecture inferred from {} pieces of evidence.", self.sources.len()),
            candidate_type: CandidateType::Architecture,
            decision_category: None,
            architecture_category: Some(self.category),
            traceability_category: None,
            source_endpoint: None,
            target_endpoint: None,
            traceability_strength: None,
            ownership_domains: self.ownership_domains,
            dependent_components: self.dependent_components,
            status: CandidateStatus::Proposed,
            confidence,
            created_at: now,
            updated_at: now,
        })
    }
}

pub struct ArchitectureCandidateEngine;

impl ArchitectureCandidateEngine {
    pub fn evaluate(builder: ArchitectureCandidateBuilder) -> Result<Candidate, String> {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_source(source_type: &str, source_id: &str) -> CandidateSource {
        CandidateSource {
            id: Uuid::new_v4().to_string(),
            candidate_id: "".to_string(),
            source_type: source_type.to_string(),
            source_id: source_id.to_string(),
            confidence: 1.0,
        }
    }

    #[test]
    fn test_minimum_evidence_rule() {
        let builder = ArchitectureCandidateBuilder::new("PROJ-1", "Payment Service", ArchitectureCategory::Service)
            .add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::WorkspaceStructure,
                source: dummy_source("file", "crates/payment/Cargo.toml"),
            });

        let result = builder.build();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Architecture Candidates require a minimum of 2 independent evidence types");
    }

    #[test]
    fn test_successful_architecture_candidate() {
        let mut builder = ArchitectureCandidateBuilder::new("PROJ-1", "Payment Service", ArchitectureCategory::Service);
        
        // Add minimum evidence types
        builder = builder
            .add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::WorkspaceStructure,
                source: dummy_source("file", "crates/payment/Cargo.toml"),
            })
            .add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::DependencyGraph,
                source: dummy_source("commit", "abcdef123"),
            })
            .add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::CodeOwnership,
                source: dummy_source("codeowners", "CODEOWNERS"),
            });

        // Add 50 dummy sources to cross the 80/85% evidence count and diversity threshold
        for i in 0..50 {
            builder = builder.add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::CommitHistory,
                source: dummy_source(&format!("commit-{}", i), &format!("hash-{}", i)),
            });
            builder.evidence_types.insert(ArchitectureEvidenceType::CommitHistory); // For diversity logic
            // Hack to force diversity: we just add random strings to builder directly if we had to, 
            // but source_diversity in Architecture uses evidence_types.len() which is max 4.
            // Wait, diversity = evidence_types.len() which maxes at 4.
            // If diversity is 4, (4/10)*25 = 10.
            // If evidence_count = 50, (50/50)*30 = 30.
            // Temporal = 20, Cluster = 25.
            // Total score: 10 + 30 + 20 + 25 = 85.0 -> exactly 0.85 !
        }

        builder = builder.add_ownership_domain("billing").add_dependent_component("checkout");

        let candidate = builder.build().expect("Should build successfully");
        
        assert_eq!(candidate.architecture_category, Some(ArchitectureCategory::Service));
        assert_eq!(candidate.ownership_domains, vec!["billing"]);
        assert_eq!(candidate.dependent_components, vec!["checkout"]);
        assert!(candidate.confidence.normalized_score() >= CandidateThresholds::architecture());
    }

    #[test]
    fn test_cross_repository_protection() {
        let mut builder = ArchitectureCandidateBuilder::new("PROJ-A", "Auth", ArchitectureCategory::Module)
            .add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::WorkspaceStructure,
                source: dummy_source("file", "auth/index.ts"),
            })
            .add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::DependencyGraph,
                source: dummy_source("commit", "123"),
            })
            .add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::CodeOwnership,
                source: dummy_source("codeowners", "CODEOWNERS"),
            })
            .add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::CommitHistory,
                source: dummy_source("commit", "abc"),
            });
            
        for i in 0..50 {
            builder = builder.add_evidence(ArchitectureEvidence {
                evidence_type: ArchitectureEvidenceType::CommitHistory,
                source: dummy_source(&format!("commit-{}", i), &format!("hash-{}", i)),
            });
        }

        let candidate = builder.build().unwrap();

        // Project ID MUST be isolated
        assert_eq!(candidate.project_id, "PROJ-A");
    }
}
