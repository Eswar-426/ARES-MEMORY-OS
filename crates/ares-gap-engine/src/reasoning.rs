use crate::models::{Gap, GapEvidence, GapReason, RootCause};
use ares_core::AresError;
use ares_decision_intelligence::storage::DecisionStore;
use ares_requirements::storage::RequirementStore;
use ares_store::Store;
use std::sync::Arc;

pub struct GapReasoner {
    store: Arc<Store>,
}

impl GapReasoner {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Evaluates a list of gaps and attaches deterministically computed reasons and evidence.
    pub fn reason(&self, mut gaps: Vec<Gap>) -> Result<Vec<Gap>, AresError> {
        let _dec_store = DecisionStore::new((*self.store).clone());
        let _req_store = RequirementStore::new((*self.store).clone());

        for gap in &mut gaps {
            match gap.gap_type {
                crate::models::GapType::MissingDecision => {
                    // Traceability Breakdown
                    let req_id = &gap.source_id;
                    let evidence = vec![GapEvidence {
                        source_entity: req_id.clone(),
                        source_type: "Requirement".to_string(),
                        explanation: "Requirement exists but has no decision links in the DB."
                            .to_string(),
                    }];

                    gap.evidence = evidence.clone();
                    gap.reason = Some(GapReason {
                        root_cause: RootCause::TraceabilityBreakdown,
                        explanation:
                            "Architectural traceability is broken between requirement and design."
                                .to_string(),
                        supporting_evidence: evidence,
                    });
                }
                crate::models::GapType::MissingEvidence => {
                    // Governance Failure
                    let dec_id = ares_core::DecisionId::from(gap.source_id.clone());
                    let evidence = vec![GapEvidence {
                        source_entity: dec_id.as_str().to_string(),
                        source_type: "Decision".to_string(),
                        explanation:
                            "Decision is approved or proposed but has 0 evidence records attached."
                                .to_string(),
                    }];

                    gap.evidence = evidence.clone();
                    gap.reason = Some(GapReason {
                        root_cause: RootCause::GovernanceFailure,
                        explanation: "Governance process was skipped. Approval happened without documented evidence.".to_string(),
                        supporting_evidence: evidence,
                    });
                }
                crate::models::GapType::MissingOwner => {
                    // Ownership Failure
                    let dec_id = ares_core::DecisionId::from(gap.source_id.clone());
                    let evidence = vec![GapEvidence {
                        source_entity: dec_id.as_str().to_string(),
                        source_type: "Decision".to_string(),
                        explanation: "Decision is marked as Approved but the owner field is NULL."
                            .to_string(),
                    }];

                    gap.evidence = evidence.clone();
                    gap.reason = Some(GapReason {
                        root_cause: RootCause::OwnershipFailure,
                        explanation: "Decisions cannot be approved without a registered owner."
                            .to_string(),
                        supporting_evidence: evidence,
                    });
                }
                crate::models::GapType::MissingImplementation => {
                    // Validation / Traceability Breakdown
                    let evidence = vec![GapEvidence {
                        source_entity: gap.source_id.clone(),
                        source_type: "Requirement".to_string(),
                        explanation: "Requirement is marked Implemented but has 0 code traces."
                            .to_string(),
                    }];

                    gap.evidence = evidence.clone();
                    gap.reason = Some(GapReason {
                        root_cause: RootCause::ValidationFailure,
                        explanation: "Status indicates completion, but there is no verifiable code implementation or test linked.".to_string(),
                        supporting_evidence: evidence,
                    });
                }
                crate::models::GapType::StaleRequirement => {
                    // Memory Decay
                    let evidence = vec![GapEvidence {
                        source_entity: gap.source_id.clone(),
                        source_type: "Requirement".to_string(),
                        explanation: "Entity has not been updated in over 6 months.".to_string(),
                    }];

                    gap.evidence = evidence.clone();
                    gap.reason = Some(GapReason {
                        root_cause: RootCause::MemoryDecay,
                        explanation: "Knowledge artifacts are decaying and lack recent curation or validation.".to_string(),
                        supporting_evidence: evidence,
                    });
                }
                crate::models::GapType::OrphanCode => {
                    let evidence = vec![GapEvidence {
                        source_entity: gap.project_id.as_str().to_string(),
                        source_type: "Project".to_string(),
                        explanation: "Detected 0 paths from decisions to code.".to_string(),
                    }];

                    gap.evidence = evidence.clone();
                    gap.reason = Some(GapReason {
                        root_cause: RootCause::TraceabilityBreakdown,
                        explanation:
                            "Code components exist without any architectural justification."
                                .to_string(),
                        supporting_evidence: evidence,
                    });
                }
            }
        }

        Ok(gaps)
    }
}
