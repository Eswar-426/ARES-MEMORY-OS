use crate::models::{ResolutionActionType, ResolutionTemplate};
use ares_gap_engine::models::{GapType, RootCause};
use uuid::Uuid;

pub struct ResolutionRuleEngine;

impl ResolutionRuleEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn get_template(&self, gap_type: &GapType, root_cause: &RootCause) -> ResolutionTemplate {
        // Deterministic mapping based on RootCause and GapType.
        // As defined by architecture guidelines.
        
        let actions = match root_cause {
            RootCause::GovernanceFailure => vec![
                ResolutionActionType::AssignOwner,
                ResolutionActionType::AddApproval,
                ResolutionActionType::AddEvidence,
            ],
            RootCause::TraceabilityBreakdown => vec![
                ResolutionActionType::CreateDecision,
                ResolutionActionType::CreateTraceabilityLink,
            ],
            RootCause::DocumentationFailure => vec![
                ResolutionActionType::UpdateDocumentation,
                ResolutionActionType::AddEvidence,
            ],
            RootCause::ValidationFailure => vec![
                ResolutionActionType::CreateValidation,
            ],
            RootCause::OwnershipFailure => vec![
                ResolutionActionType::AssignOwner,
            ],
            RootCause::MemoryDecay => vec![
                ResolutionActionType::ReviewEntity,
                ResolutionActionType::UpdateDocumentation,
            ],
            RootCause::ProcessDrift => vec![
                ResolutionActionType::GovernanceReview,
            ],
        };

        // Customise template title based on GapType
        let title = match gap_type {
            GapType::MissingDecision => "Decision Traceability Recovery".to_string(),
            GapType::MissingEvidence => "Decision Evidence Governance".to_string(),
            GapType::MissingImplementation => "Implementation Traceability Recovery".to_string(),
            GapType::MissingOwner => "Ownership Recovery".to_string(),
            GapType::OrphanCode => "Orphaned Artifact Resolution".to_string(),
            GapType::StaleRequirement => "Requirement Refresh and Review".to_string(),
        };

        // Determine base estimated health gain and debt reduction based on gap type and root cause
        // These will be scaled later by the simulator
        let (base_health_gain, base_debt_reduction) = match gap_type {
            GapType::MissingDecision => (15.0, 10.0),
            GapType::MissingEvidence => (10.0, 8.0),
            GapType::MissingImplementation => (12.0, 10.0),
            GapType::MissingOwner => (8.0, 5.0),
            GapType::OrphanCode => (20.0, 15.0),
            GapType::StaleRequirement => (5.0, 5.0),
        };

        ResolutionTemplate {
            id: Uuid::now_v7().to_string(),
            title,
            actions,
            expected_health_gain: base_health_gain,
            expected_debt_reduction: base_debt_reduction,
        }
    }
}

impl Default for ResolutionRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}
