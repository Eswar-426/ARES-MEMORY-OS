#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::models::{ResolutionActionType, ResolutionCategory};
    use crate::rules::ResolutionRuleEngine;
    use crate::simulator::MemoryHealthSimulator;
    use ares_gap_engine::models::{GapType, RootCause};

    #[test]
    fn test_rule_engine_governance_failure() {
        let engine = ResolutionRuleEngine::new();
        let gap_type = GapType::MissingEvidence;
        let root_cause = RootCause::GovernanceFailure;

        let template = engine.get_template(&gap_type, &root_cause);

        assert_eq!(template.title, "Decision Evidence Governance");
        assert!(template
            .actions
            .contains(&ResolutionActionType::AssignOwner));
        assert!(template
            .actions
            .contains(&ResolutionActionType::AddApproval));
        assert!(template
            .actions
            .contains(&ResolutionActionType::AddEvidence));
        assert_eq!(template.expected_health_gain, 10.0);
        assert_eq!(template.expected_debt_reduction, 8.0);
    }

    #[test]
    fn test_simulator_infer_category() {
        let simulator = MemoryHealthSimulator::new();
        assert_eq!(
            simulator.infer_category(&GapType::MissingDecision),
            ResolutionCategory::Traceability
        );
        assert_eq!(
            simulator.infer_category(&GapType::MissingOwner),
            ResolutionCategory::Ownership
        );
        assert_eq!(
            simulator.infer_category(&GapType::StaleRequirement),
            ResolutionCategory::Documentation
        );
        assert_eq!(
            simulator.infer_category(&GapType::MissingEvidence),
            ResolutionCategory::Governance
        );
    }

    #[test]
    fn test_simulator_simulate() {
        let engine = ResolutionRuleEngine::new();
        let gap_type = GapType::MissingDecision;
        let root_cause = RootCause::TraceabilityBreakdown;
        let template = engine.get_template(&gap_type, &root_cause);

        let simulator = MemoryHealthSimulator::new();
        let (gain, debt, breakdown) = simulator.simulate(&gap_type, &template);

        assert_eq!(gain, 15.0);
        assert_eq!(debt, 10.0);
        assert_eq!(breakdown.traceability_gain, 15.0 * 0.8);
        assert_eq!(breakdown.governance_gain, 15.0 * 0.2);
    }
}
