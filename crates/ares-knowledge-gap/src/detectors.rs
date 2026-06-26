use crate::models::{
    GapEvidence, GapSeverity, KnowledgeGap, KnowledgeGapType, RemediationRecommendation,
};
use ares_core::types::node::NodeType;
use ares_core::{AresError, EdgeDirection, EdgeType, ProjectId};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;

pub struct KnowledgeGapDetector<'a> {
    retrieval_engine: &'a MemoryRetrievalEngine,
}

impl<'a> KnowledgeGapDetector<'a> {
    pub fn new(retrieval_engine: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval_engine }
    }

    pub fn scan_project(&self, project_id: &ProjectId) -> Result<Vec<KnowledgeGap>, AresError> {
        let mut gaps = Vec::new();

        gaps.extend(self.detect_missing_requirements(project_id)?);
        gaps.extend(self.detect_missing_decisions(project_id)?);
        gaps.extend(self.detect_missing_architecture(project_id)?);
        gaps.extend(self.detect_missing_ownership(project_id)?);
        gaps.extend(self.detect_missing_tests(project_id)?);
        gaps.extend(self.detect_missing_runtime(project_id)?);
        gaps.extend(self.detect_missing_outcome(project_id)?);
        gaps.extend(self.detect_missing_traceability(project_id)?);
        gaps.extend(self.detect_knowledge_blind_spots(project_id)?);

        Ok(gaps)
    }

    fn detect_missing_requirements(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<KnowledgeGap>, AresError> {
        let decisions = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::Decision)?;
        let mut gaps = Vec::new();

        for dec in decisions {
            let upstream = self.retrieval_engine.get_neighborhood(
                dec.id.as_ref(),
                EdgeDirection::Incoming,
                &[EdgeType::Drives],
            )?;

            let has_req = upstream
                .iter()
                .any(|n| n.node_type == NodeType::Requirement);
            if !has_req {
                gaps.push(KnowledgeGap {
                    gap_type: KnowledgeGapType::MissingRequirement,
                    severity: GapSeverity::High,
                    evidence: GapEvidence {
                        source_nodes: vec![dec.id.to_string()],
                        missing_nodes: vec![NodeType::Requirement],
                        rationale: format!(
                            "Decision {} exists without an upstream Requirement.",
                            dec.label
                        ),
                    },
                    remediation: RemediationRecommendation {
                        priority: GapSeverity::High,
                        owner: None, // Could infer from dec owners
                        recommended_action: "Document the Requirement that drove this Decision."
                            .to_string(),
                    },
                });
            }
        }
        Ok(gaps)
    }

    fn detect_missing_decisions(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<KnowledgeGap>, AresError> {
        let architectures = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::Architecture)?;
        let mut gaps = Vec::new();

        for arch in architectures {
            let upstream = self.retrieval_engine.get_neighborhood(
                arch.id.as_ref(),
                EdgeDirection::Incoming,
                &[EdgeType::Drives],
            )?;

            let has_dec = upstream.iter().any(|n| n.node_type == NodeType::Decision);
            if !has_dec {
                gaps.push(KnowledgeGap {
                    gap_type: KnowledgeGapType::MissingDecision,
                    severity: GapSeverity::High,
                    evidence: GapEvidence {
                        source_nodes: vec![arch.id.to_string()],
                        missing_nodes: vec![NodeType::Decision],
                        rationale: format!(
                            "Architecture {} exists without a justifying Decision.",
                            arch.label
                        ),
                    },
                    remediation: RemediationRecommendation {
                        priority: GapSeverity::High,
                        owner: None,
                        recommended_action:
                            "Record the Decision (e.g. ADR) that led to this Architecture."
                                .to_string(),
                    },
                });
            }
        }
        Ok(gaps)
    }

    fn detect_missing_architecture(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<KnowledgeGap>, AresError> {
        let code_nodes = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::File)?;
        let mut gaps = Vec::new();

        for code in code_nodes {
            let upstream = self.retrieval_engine.get_neighborhood(
                code.id.as_ref(),
                EdgeDirection::Incoming,
                &[EdgeType::Drives, EdgeType::Implements, EdgeType::Contains],
            )?;

            let has_dec = upstream.iter().any(|n| n.node_type == NodeType::Decision);
            let has_arch = upstream
                .iter()
                .any(|n| n.node_type == NodeType::Architecture);

            if has_dec && !has_arch {
                gaps.push(KnowledgeGap {
                    gap_type: KnowledgeGapType::MissingArchitecture,
                    severity: GapSeverity::Medium,
                    evidence: GapEvidence {
                        source_nodes: vec![code.id.to_string()],
                        missing_nodes: vec![NodeType::Architecture],
                        rationale: format!("Code {} is driven directly by a Decision, skipping Architecture design.", code.label),
                    },
                    remediation: RemediationRecommendation {
                        priority: GapSeverity::Medium,
                        owner: None,
                        recommended_action: "Insert an Architecture node to define the design translating the Decision to Code.".to_string(),
                    },
                });
            }
        }
        Ok(gaps)
    }

    fn detect_missing_ownership(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<KnowledgeGap>, AresError> {
        let features = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::Feature)?;
        let mut gaps = Vec::new();

        for feat in features {
            let has_owner = feat
                .properties
                .get("owners")
                .is_some_and(|v| v.as_array().is_some_and(|arr| !arr.is_empty()));
            if !has_owner {
                gaps.push(KnowledgeGap {
                    gap_type: KnowledgeGapType::MissingOwnership,
                    severity: GapSeverity::Critical,
                    evidence: GapEvidence {
                        source_nodes: vec![feat.id.to_string()],
                        missing_nodes: vec![NodeType::Team, NodeType::Person],
                        rationale: format!(
                            "Capability/Feature {} lacks an explicit owner.",
                            feat.label
                        ),
                    },
                    remediation: RemediationRecommendation {
                        priority: GapSeverity::Critical,
                        owner: None,
                        recommended_action: "Assign a Team or Person owner to this Capability."
                            .to_string(),
                    },
                });
            }
        }
        Ok(gaps)
    }

    fn detect_missing_tests(&self, project_id: &ProjectId) -> Result<Vec<KnowledgeGap>, AresError> {
        let code_nodes = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::File)?;
        let mut gaps = Vec::new();

        for code in code_nodes {
            let downstream = self.retrieval_engine.get_neighborhood(
                code.id.as_ref(),
                EdgeDirection::Incoming,
                &[EdgeType::ValidatedBy],
            )?;

            // Tests are usually incoming 'Tests' edge to Code
            // Wait, or outgoing from Test to Code? Usually Test -> Tests -> Code. So Incoming to Code.
            if downstream.is_empty() {
                // To avoid spamming, only for important code, but we'll flag it for now
                gaps.push(KnowledgeGap {
                    gap_type: KnowledgeGapType::MissingTests,
                    severity: GapSeverity::Medium,
                    evidence: GapEvidence {
                        source_nodes: vec![code.id.to_string()],
                        missing_nodes: vec![],
                        rationale: format!("Code {} has no test coverage mapped.", code.label),
                    },
                    remediation: RemediationRecommendation {
                        priority: GapSeverity::Medium,
                        owner: None,
                        recommended_action:
                            "Write unit or integration tests for this code and link them."
                                .to_string(),
                    },
                });
            }
        }
        Ok(gaps)
    }

    fn detect_missing_runtime(
        &self,
        _project_id: &ProjectId,
    ) -> Result<Vec<KnowledgeGap>, AresError> {
        // Simplified for brevity, assume we look for NodeType::Release without Runtime
        Ok(Vec::new())
    }

    fn detect_missing_outcome(
        &self,
        _project_id: &ProjectId,
    ) -> Result<Vec<KnowledgeGap>, AresError> {
        // Simplified
        Ok(Vec::new())
    }

    fn detect_missing_traceability(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<KnowledgeGap>, AresError> {
        // Find artifacts completely disconnected
        let all_nodes = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::File)?;
        let mut gaps = Vec::new();

        for node in all_nodes {
            let in_edges = self.retrieval_engine.get_all_edges_to(node.id.as_ref())?;
            let out_edges = self.retrieval_engine.get_all_edges_from(node.id.as_ref())?;

            if in_edges.is_empty() && out_edges.is_empty() {
                gaps.push(KnowledgeGap {
                    gap_type: KnowledgeGapType::MissingTraceability,
                    severity: GapSeverity::High,
                    evidence: GapEvidence {
                        source_nodes: vec![node.id.to_string()],
                        missing_nodes: vec![],
                        rationale: format!("Node {} is completely isolated with no incoming or outgoing traceability edges.", node.label),
                    },
                    remediation: RemediationRecommendation {
                        priority: GapSeverity::High,
                        owner: None,
                        recommended_action: "Map this artifact to its driving requirements or architectures.".to_string(),
                    },
                });
            }
        }
        Ok(gaps)
    }

    fn detect_knowledge_blind_spots(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<KnowledgeGap>, AresError> {
        let features = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::Feature)?;
        let mut gaps = Vec::new();

        for feat in features {
            let in_edges = self.retrieval_engine.get_all_edges_to(feat.id.as_ref())?;
            let out_edges = self.retrieval_engine.get_all_edges_from(feat.id.as_ref())?;

            let connectivity_weight = (in_edges.len() + out_edges.len()) as f32;
            let ownership_risk = if feat.properties.get("owners").is_some() {
                0.1
            } else {
                1.0
            };
            let traceability_risk = if connectivity_weight > 0.0 { 0.2 } else { 1.0 };
            let drift_risk = if feat
                .properties
                .get("has_drift")
                .is_some_and(|v| v.as_bool().unwrap_or(false))
            {
                1.0
            } else {
                0.1
            };
            let completeness_risk = 0.5;

            let blind_spot_score = connectivity_weight
                * ownership_risk
                * traceability_risk
                * drift_risk
                * completeness_risk;

            // Threshold for blind spot
            if blind_spot_score > 5.0 {
                gaps.push(KnowledgeGap {
                    gap_type: KnowledgeGapType::KnowledgeBlindSpot,
                    severity: GapSeverity::Critical,
                    evidence: GapEvidence {
                        source_nodes: vec![feat.id.to_string()],
                        missing_nodes: vec![],
                        rationale: format!("Feature {} has a high blind spot score of {}.", feat.label, blind_spot_score),
                    },
                    remediation: RemediationRecommendation {
                        priority: GapSeverity::Critical,
                        owner: None,
                        recommended_action: "Address missing ownership, resolve drift, or improve documentation traceability.".to_string(),
                    },
                });
            }
        }
        Ok(gaps)
    }
}
