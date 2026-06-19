use ares_governance::dashboard::DashboardGenerator;
use ares_governance::models::{GovernanceCertification, GovernanceScorecard, ViolationSeverity, ComplianceViolation, CertificationLevel, EnforcementAction, PolicyCategory, RequirementDriftSummary};
use ares_requirements::{RequirementCoverageSummary, RequirementCoverageTrend, GapSummary};


#[test]
fn test_dashboard_aggregation() {
    let certification = GovernanceCertification {
        id: "cert".to_string(),
        project_id: "test".to_string(),
        certified: true,
        violations_count: 2,
        policy_score: 100.0,
        level: CertificationLevel::Gold,
        scorecard: GovernanceScorecard {
            ownership_score: 1.0,
            traceability_score: 1.0,
            evidence_score: 1.0,
            approval_score: 1.0,
            retention_score: 1.0,
            security_score: 1.0,
            architecture_score: 1.0,
            overall_score: 1.0,
        },
        evaluated_at: 0,
    };

    let violations = vec![
        ComplianceViolation {
            id: "V1".to_string(),
            severity: ViolationSeverity::Warning,
            policy_name: "P1".to_string(),
            node_id: "REQ-1".to_string(),
            reason: "warn".to_string(),
            supporting_nodes: vec![],
            enforcement: EnforcementAction::Warn,
            category: PolicyCategory::Architecture,
        },
        ComplianceViolation {
            id: "V2".to_string(),
            severity: ViolationSeverity::Critical,
            policy_name: "P2".to_string(),
            node_id: "REQ-2".to_string(),
            reason: "crit".to_string(),
            supporting_nodes: vec![],
            enforcement: EnforcementAction::Warn,
            category: PolicyCategory::Architecture,
        },
    ];

    let coverage_summary = RequirementCoverageSummary {
        total_requirements: 10,
        fully_covered: 5,
        partially_covered: 3,
        orphaned: 2,
        average_coverage: 65.0,
    };

    let coverage_trend = RequirementCoverageTrend {
        previous_coverage: 60.0,
        current_coverage: 65.0,
        delta: 5.0,
    };

    let drift_summary = RequirementDriftSummary {
        structural_drift: 1,
        semantic_drift: 0,
        critical_drift: 1,
        unresolved_drift: 1,
    };

    let top_gaps = vec![
        GapSummary {
            gap_type: ares_requirements::KnowledgeGapType::MissingDecision,
            count: 2,
        },
    ];

    let evolution = ares_governance::models::EvolutionMetrics {
        total_requirement_events: 5,
        requirements_changed_this_week: 2,
        requirements_regressed: 0,
        requirements_improved: 1,
    };

    let dashboard = DashboardGenerator::generate_dashboard(
        &certification,
        violations,
        coverage_summary,
        coverage_trend,
        drift_summary,
        evolution,
        top_gaps,
        &[], // knowledge_gaps
    );

    assert_eq!(dashboard.requirement_coverage.fully_covered, 5);
    assert_eq!(dashboard.requirement_drift.critical_drift, 1);
    assert_eq!(dashboard.top_gaps.len(), 1);
    
    // Check sorting of violations (Critical should be first)
    assert_eq!(dashboard.top_violations[0].severity, ViolationSeverity::Critical);
    assert_eq!(dashboard.top_violations[1].severity, ViolationSeverity::Warning);
}
