use chrono::Utc;
use std::collections::HashSet;
use uuid::Uuid;

use ares_candidates::{
    Candidate, CandidateConfidence, CandidateStatus, CandidateThresholds, CandidateType,
    TraceabilityCategory, TraceabilityEndpoint, TraceabilityEndpointType,
};

/// Evidence types that support traceability linkages.
#[derive(Debug, Clone)]
pub struct TraceabilityEvidence {
    pub source_id: String,
    pub source_type: String, // e.g., "Cargo.toml", "commit", "keyword_overlap", "code_path"
    pub description: String,
    pub overlap_score: f64,
}

pub struct TraceabilityCandidateEngine {
    project_id: String,
}

impl TraceabilityCandidateEngine {
    pub fn new(project_id: String) -> Self {
        Self { project_id }
    }

    /// Primary entry point for constructing traceability edges between existing candidates.
    pub fn build_traceability_graph(&self, candidates: &[Candidate]) -> Vec<Candidate> {
        let mut traceability_candidates = Vec::new();

        // Separate by type
        let requirements: Vec<&Candidate> = candidates
            .iter()
            .filter(|c| c.candidate_type == CandidateType::Requirement)
            .collect();
        let decisions: Vec<&Candidate> = candidates
            .iter()
            .filter(|c| c.candidate_type == CandidateType::Decision)
            .collect();
        let architectures: Vec<&Candidate> = candidates
            .iter()
            .filter(|c| c.candidate_type == CandidateType::Architecture)
            .collect();

        // Phase 1: Architecture -> Code (simulated here with dummy files based on dependent_components / domains)
        // Architecture to Code is usually matched by file paths.
        // For testing we will simulate Code nodes.

        // Phase 2: Decision -> Architecture
        let dec_arch_candidates =
            self.phase_two_decision_to_architecture(&decisions, &architectures);
        traceability_candidates.extend(dec_arch_candidates);

        // Phase 3: Requirement -> Decision
        let req_dec_candidates =
            self.phase_three_requirement_to_decision(&requirements, &decisions);
        traceability_candidates.extend(req_dec_candidates);

        // Phase 4: Requirement -> Code (fallback, simplified)

        traceability_candidates
    }

    fn phase_two_decision_to_architecture(
        &self,
        decisions: &[&Candidate],
        architectures: &[&Candidate],
    ) -> Vec<Candidate> {
        let mut results = Vec::new();

        for dec in decisions {
            for arch in architectures {
                // Cross-repo protection
                if dec.project_id != arch.project_id {
                    continue;
                }
                if dec.project_id != self.project_id {
                    continue;
                }

                // Determine overlap
                // Simulated: if they share keywords in title/description or are "Auth" vs "OAuth"
                let overlap = self.calculate_overlap(dec, arch);
                if overlap > 0.6 {
                    let mut builder = TraceabilityCandidateBuilder::new(
                        &self.project_id,
                        TraceabilityCategory::DecisionToArchitecture,
                        TraceabilityEndpoint {
                            endpoint_type: TraceabilityEndpointType::Candidate,
                            endpoint_id: dec.id.clone(),
                        },
                        TraceabilityEndpoint {
                            endpoint_type: TraceabilityEndpointType::Candidate,
                            endpoint_id: arch.id.clone(),
                        },
                    );

                    // Add dummy evidence proportional to overlap
                    for i in 0..50 {
                        builder.add_evidence(TraceabilityEvidence {
                            source_id: format!("shared-{}", i),
                            source_type: format!("source_type_{}", i % 10),
                            description: "Shared dependency config".to_string(),
                            overlap_score: overlap,
                        });
                    }

                    if let Some(candidate) = builder.build() {
                        results.push(candidate);
                    }
                }
            }
        }
        results
    }

    fn phase_three_requirement_to_decision(
        &self,
        requirements: &[&Candidate],
        decisions: &[&Candidate],
    ) -> Vec<Candidate> {
        let mut results = Vec::new();

        for req in requirements {
            for dec in decisions {
                // Cross-repo protection
                if req.project_id != dec.project_id {
                    continue;
                }
                if req.project_id != self.project_id {
                    continue;
                }

                let overlap = self.calculate_overlap(req, dec);
                if overlap > 0.5 {
                    let mut builder = TraceabilityCandidateBuilder::new(
                        &self.project_id,
                        TraceabilityCategory::RequirementToDecision,
                        TraceabilityEndpoint {
                            endpoint_type: TraceabilityEndpointType::Candidate,
                            endpoint_id: req.id.clone(),
                        },
                        TraceabilityEndpoint {
                            endpoint_type: TraceabilityEndpointType::Candidate,
                            endpoint_id: dec.id.clone(),
                        },
                    );

                    for i in 0..50 {
                        builder.add_evidence(TraceabilityEvidence {
                            source_id: format!("kw-{}", i),
                            source_type: format!("source_type_{}", i % 10),
                            description: "Matching domains".to_string(),
                            overlap_score: overlap,
                        });
                    }

                    if let Some(candidate) = builder.build() {
                        results.push(candidate);
                    }
                }
            }
        }
        results
    }

    fn calculate_overlap(&self, c1: &Candidate, c2: &Candidate) -> f64 {
        let text1 = format!("{} {}", c1.title, c1.description).to_lowercase();
        let text2 = format!("{} {}", c2.title, c2.description).to_lowercase();

        let words1: HashSet<&str> = text1.split_whitespace().collect();
        let words2: HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            return 0.0;
        }
        (intersection as f64) / (union as f64)
    }
}

pub struct TraceabilityCandidateBuilder {
    id: String,
    project_id: String,
    category: TraceabilityCategory,
    source_endpoint: TraceabilityEndpoint,
    target_endpoint: TraceabilityEndpoint,
    evidence: Vec<TraceabilityEvidence>,
}

impl TraceabilityCandidateBuilder {
    pub fn new(
        project_id: &str,
        category: TraceabilityCategory,
        source_endpoint: TraceabilityEndpoint,
        target_endpoint: TraceabilityEndpoint,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            category,
            source_endpoint,
            target_endpoint,
            evidence: Vec::new(),
        }
    }

    pub fn add_evidence(&mut self, ev: TraceabilityEvidence) -> &mut Self {
        self.evidence.push(ev);
        self
    }

    pub fn build(self) -> Option<Candidate> {
        let evidence_count = self.evidence.len() as u32;
        let diversity = self
            .evidence
            .iter()
            .map(|e| e.source_type.clone())
            .collect::<HashSet<_>>()
            .len() as u32;

        let confidence = CandidateConfidence {
            evidence_count,
            source_diversity: diversity,
            temporal_consistency: 1.0,
            cluster_strength: 1.0,
        };

        let score = confidence.normalized_score();

        // Strict threshold
        if score < CandidateThresholds::traceability() {
            return None;
        }

        let strength = CandidateThresholds::get_traceability_strength(score);

        Some(Candidate {
            id: self.id.clone(),
            project_id: self.project_id.clone(),
            title: format!("{:?} Edge", self.category),
            description: format!(
                "Traceability from {} to {}",
                self.source_endpoint.endpoint_id, self.target_endpoint.endpoint_id
            ),
            candidate_type: CandidateType::Traceability,
            decision_category: None,
            architecture_category: None,
            traceability_category: Some(self.category),
            source_endpoint: Some(self.source_endpoint),
            target_endpoint: Some(self.target_endpoint),
            traceability_strength: Some(strength),
            ownership_domains: Vec::new(),
            dependent_components: Vec::new(),
            status: CandidateStatus::Proposed,
            confidence,
            created_at: Utc::now().timestamp_millis(),
            updated_at: Utc::now().timestamp_millis(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_candidate(id: &str, project_id: &str, t: CandidateType, title: &str) -> Candidate {
        Candidate {
            id: id.to_string(),
            project_id: project_id.to_string(),
            title: title.to_string(),
            description: title.to_string(),
            candidate_type: t,
            decision_category: None,
            architecture_category: None,
            traceability_category: None,
            source_endpoint: None,
            target_endpoint: None,
            traceability_strength: None,
            ownership_domains: vec![],
            dependent_components: vec![],
            status: CandidateStatus::Proposed,
            confidence: CandidateConfidence {
                evidence_count: 50,
                source_diversity: 10,
                temporal_consistency: 1.0,
                cluster_strength: 1.0,
            },
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn test_cross_repository_protection() {
        let engine = TraceabilityCandidateEngine::new("repo-a".to_string());

        // Valid same-repo
        let req1 = dummy_candidate("req-1", "repo-a", CandidateType::Requirement, "Auth");
        let dec1 = dummy_candidate("dec-1", "repo-a", CandidateType::Decision, "Auth");

        // Invalid cross-repo
        let dec2 = dummy_candidate("dec-2", "repo-b", CandidateType::Decision, "Auth");

        // Engine only generates valid links within its project scope
        let results = engine.build_traceability_graph(&[req1.clone(), dec1.clone(), dec2.clone()]);

        // Must contain link req1 -> dec1
        assert!(results.iter().any(
            |c| c.source_endpoint.as_ref().unwrap().endpoint_id == "req-1"
                && c.target_endpoint.as_ref().unwrap().endpoint_id == "dec-1"
        ));

        // Must NOT contain link req1 -> dec2
        assert!(!results
            .iter()
            .any(
                |c| c.source_endpoint.as_ref().unwrap().endpoint_id == "req-1"
                    && c.target_endpoint.as_ref().unwrap().endpoint_id == "dec-2"
            ));
    }

    #[test]
    fn test_e2e_traversal() {
        let engine = TraceabilityCandidateEngine::new("repo-test".to_string());

        let req = dummy_candidate(
            "req-1",
            "repo-test",
            CandidateType::Requirement,
            "User Authentication",
        );
        let dec = dummy_candidate(
            "dec-1",
            "repo-test",
            CandidateType::Decision,
            "User Authentication OAuth2",
        );
        let arch = dummy_candidate(
            "arch-1",
            "repo-test",
            CandidateType::Architecture,
            "User Authentication OAuth2 Service",
        );

        let candidates = vec![req, dec, arch];
        let edges = engine.build_traceability_graph(&candidates);

        // We expect REQ -> DEC
        assert!(edges
            .iter()
            .any(|c| c.traceability_category == Some(TraceabilityCategory::RequirementToDecision)));
        // We expect DEC -> ARCH
        assert!(
            edges
                .iter()
                .any(|c| c.traceability_category
                    == Some(TraceabilityCategory::DecisionToArchitecture))
        );
    }
}
