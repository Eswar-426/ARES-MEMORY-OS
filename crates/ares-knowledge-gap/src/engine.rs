use crate::detectors::KnowledgeGapDetector;
use crate::models::{
    GapEvidence, GapSeverity, KnowledgeGap, KnowledgeGapType, RemediationRecommendation,
};
use ares_core::{AresError, ProjectId};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;

pub struct KnowledgeGapEngine<'a> {
    #[allow(dead_code)]
    retrieval_engine: &'a MemoryRetrievalEngine,
    detector: KnowledgeGapDetector<'a>,
}

impl<'a> KnowledgeGapEngine<'a> {
    pub fn new(retrieval_engine: &'a MemoryRetrievalEngine) -> Self {
        Self {
            retrieval_engine,
            detector: KnowledgeGapDetector::new(retrieval_engine),
        }
    }

    pub fn scan_and_analyze(&self, project_id: &ProjectId) -> Result<Vec<KnowledgeGap>, AresError> {
        let mut gaps = self.detector.scan_project(project_id)?;

        // Compute Knowledge Debt at the repository level based on findings
        let mut drift_score = 0.0;
        let mut staleness_score = 0.0;
        let mut gov_penalty = 0.0;
        let mut comp_penalty = 0.0;
        let mut trace_penalty = 0.0;

        for gap in &gaps {
            match gap.gap_type {
                KnowledgeGapType::MissingOwnership | KnowledgeGapType::MissingDecision => {
                    gov_penalty += 5.0
                }
                KnowledgeGapType::MissingRequirement | KnowledgeGapType::MissingArchitecture => {
                    comp_penalty += 5.0
                }
                KnowledgeGapType::MissingTraceability => trace_penalty += 10.0,
                _ => {}
            }
        }

        // Mock drift & staleness retrieval
        // In reality, we'd query properties for drift and compute time-based staleness.
        drift_score += 10.0;
        staleness_score += 5.0;

        let debt_score = drift_score + staleness_score + gov_penalty + comp_penalty + trace_penalty;

        if debt_score > 0.0 {
            gaps.push(KnowledgeGap {
                gap_type: KnowledgeGapType::KnowledgeDebt,
                severity: if debt_score > 50.0 {
                    GapSeverity::Critical
                } else {
                    GapSeverity::High
                },
                evidence: GapEvidence {
                    source_nodes: vec![],
                    missing_nodes: vec![],
                    rationale: format!(
                        "Repository has accrued a Knowledge Debt score of {}.",
                        debt_score
                    ),
                },
                remediation: RemediationRecommendation {
                    priority: GapSeverity::High,
                    owner: None,
                    recommended_action:
                        "Dedicate a sprint to paying down memory debt by fixing structural gaps."
                            .to_string(),
                },
            });
        }

        Ok(gaps)
    }
}
