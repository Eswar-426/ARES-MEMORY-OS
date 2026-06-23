use crate::models::*;

pub struct DashboardGenerator;

impl DashboardGenerator {
    pub fn generate_dashboard(
        certification: &GovernanceCertification,
        top_violations: Vec<ComplianceViolation>,
        requirement_coverage: ares_requirements::RequirementCoverageSummary,
        requirement_coverage_trend: ares_requirements::RequirementCoverageTrend,
        requirement_drift: RequirementDriftSummary,
        evolution: EvolutionMetrics,
        top_gaps: Vec<ares_requirements::GapSummary>,
        knowledge_gaps: &[ares_requirements::KnowledgeGap],
    ) -> GovernanceDashboard {
        let mut violations = top_violations;
        // Sort violations by severity (Critical > Error > Warning > Info)
        violations.sort_by(|a, b| {
            let rank = |s: &ViolationSeverity| match s {
                ViolationSeverity::Critical => 0,
                ViolationSeverity::Error => 1,
                ViolationSeverity::Warning => 2,
                ViolationSeverity::Info => 3,
            };
            rank(&a.severity).cmp(&rank(&b.severity))
        });

        let top_10 = violations.into_iter().take(10).collect();

        let mut orphan_requirements = 0;
        let mut orphan_decisions = 0;
        let mut missing_owners = 0;
        let mut missing_evidence = 0;
        let mut traceability_gaps = 0;

        for gap in knowledge_gaps {
            match gap.gap_type {
                ares_requirements::KnowledgeGapType::UnapprovedRequirement => {
                    orphan_requirements += 1
                }
                ares_requirements::KnowledgeGapType::OrphanedDecision => orphan_decisions += 1,
                ares_requirements::KnowledgeGapType::MissingOwner => missing_owners += 1,
                ares_requirements::KnowledgeGapType::MissingTest => missing_evidence += 1,
                ares_requirements::KnowledgeGapType::MissingImplementation
                | ares_requirements::KnowledgeGapType::MissingDecision => traceability_gaps += 1,
                _ => {}
            }
        }

        GovernanceDashboard {
            requirement_coverage,
            requirement_coverage_trend,
            requirement_drift,
            evolution,
            top_gaps,
            certification: certification.clone(),
            scorecard: certification.scorecard.clone(),
            decision_health: DecisionHealthMetrics {
                total_decisions: 0,
                active_decisions: 0,
                stale_decisions: 0,
                expired_decisions: 0,
                orphan_decisions,
                health_score: 1.0,
            },
            knowledge_debt: KnowledgeDebtMetrics {
                orphan_requirements,
                orphan_decisions,
                missing_owners,
                missing_evidence,
                traceability_gaps,
                policy_violations: certification.violations_count,
                debt_score: (orphan_requirements + orphan_decisions + traceability_gaps) as f32
                    * 5.0,
            },
            approvals: ApprovalMetrics {
                pending: 0,
                approved_today: 0,
                rejected_today: 0,
                expired: 0,
            },
            compliance_drift: DriftMetrics {
                drift_detected: false,
                changed_policies: 0,
                unevaluated_policies: 0,
            },
            top_violations: top_10,
        }
    }
}
