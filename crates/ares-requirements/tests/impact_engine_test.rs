use ares_requirements::impact::{ChangeRisk, ImpactSeverity, RequirementImpactEngine};
use ares_traceability::{test_utils::TestGraphBuilder, TraceTargetType};

#[test]
fn test_impact_engine_small_graph() {
    let graph = TestGraphBuilder::new()
        .link_rel("REQ-1", "DEC-1", TraceTargetType::Decision, "Satisfies")
        .link_rel("DEC-1", "CODE-1", TraceTargetType::Code, "Implements")
        .build();

    let engine = RequirementImpactEngine::new(&graph);
    let report = engine.evaluate_impact("REQ-1");

    assert_eq!(report.affected_decisions.len(), 1);
    assert_eq!(report.affected_code.len(), 1);

    // Score: 1 Decision (10) + 1 Code (2) = 12
    assert_eq!(report.blast_radius_score, 12.0);
    assert_eq!(report.severity, ImpactSeverity::Low);
    assert_eq!(report.risk, ChangeRisk::High); // Risk += 40 for Decision
}

#[test]
fn test_impact_engine_large_graph() {
    let graph = TestGraphBuilder::new()
        .link_rel("REQ-2", "DEC-1", TraceTargetType::Decision, "Satisfies")
        .link_rel("REQ-2", "DEC-2", TraceTargetType::Decision, "Satisfies")
        .link_rel("REQ-2", "DEC-3", TraceTargetType::Decision, "Satisfies")
        // 10 codes
        .link_rel("DEC-1", "CODE-1", TraceTargetType::Code, "Implements")
        .link_rel("DEC-1", "CODE-2", TraceTargetType::Code, "Implements")
        .link_rel("DEC-1", "CODE-3", TraceTargetType::Code, "Implements")
        .link_rel("DEC-2", "CODE-4", TraceTargetType::Code, "Implements")
        .link_rel("DEC-2", "CODE-5", TraceTargetType::Code, "Implements")
        .link_rel("DEC-2", "CODE-6", TraceTargetType::Code, "Implements")
        .link_rel("DEC-3", "CODE-7", TraceTargetType::Code, "Implements")
        .link_rel("DEC-3", "CODE-8", TraceTargetType::Code, "Implements")
        .link_rel("DEC-3", "CODE-9", TraceTargetType::Code, "Implements")
        .link_rel("DEC-3", "CODE-10", TraceTargetType::Code, "Implements")
        // 5 tests
        .link_rel("CODE-1", "TEST-1", TraceTargetType::Test, "Validates")
        .link_rel("CODE-2", "TEST-2", TraceTargetType::Test, "Validates")
        .link_rel("CODE-4", "TEST-3", TraceTargetType::Test, "Validates")
        .link_rel("CODE-7", "TEST-4", TraceTargetType::Test, "Validates")
        .link_rel("CODE-10", "TEST-5", TraceTargetType::Test, "Validates")
        .build();

    let engine = RequirementImpactEngine::new(&graph);
    let report = engine.evaluate_impact("REQ-2");

    assert_eq!(report.affected_decisions.len(), 3);
    assert_eq!(report.affected_code.len(), 10);
    assert_eq!(report.affected_tests.len(), 5);

    // Score: 3 Dec (30) + 10 Code (20) + 5 Test (20) = 70
    assert_eq!(report.blast_radius_score, 70.0);
    assert_eq!(report.severity, ImpactSeverity::High);
}

#[test]
fn test_impact_engine_governance_dependency() {
    let graph = TestGraphBuilder::new()
        .link_rel(
            "REQ-3",
            "GOV-1",
            TraceTargetType::Governance,
            "CompliesWith",
        )
        .link_rel("REQ-3", "CODE-1", TraceTargetType::Code, "Implements")
        .build();

    let engine = RequirementImpactEngine::new(&graph);
    let report = engine.evaluate_impact("REQ-3");

    // Score: 1 Gov (6) + 1 Code (2) = 8
    assert_eq!(report.blast_radius_score, 8.0);
    assert_eq!(report.severity, ImpactSeverity::Low);
    assert_eq!(report.risk, ChangeRisk::Medium); // Gov risk += 30 -> 30 (Medium)
}
