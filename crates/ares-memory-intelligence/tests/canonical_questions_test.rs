use ares_memory_intelligence::facade::MemoryFacade;
use ares_requirements::impact::{RequirementImpactReport, ImpactSeverity, ChangeRisk};

#[test]
fn test_q16_impact_snapshot() {
    let report = RequirementImpactReport {
        requirement_id: "REQ-1".to_string(),
        affected_decisions: vec!["DEC-1".to_string()],
        affected_architecture: vec![],
        affected_code: vec!["CODE-1".to_string()],
        affected_tests: vec!["TEST-1".to_string()],
        affected_runtime_metrics: vec![],
        affected_governance: vec!["GOV-1".to_string()],
        blast_radius_score: 22.0,
        severity: ImpactSeverity::Low,
        risk: ChangeRisk::Medium,
        impact_breakdown: vec![],
    };

    let md = MemoryFacade::format_impact_report(&report);

    let expected = "\
Requirement: REQ-1

Blast Radius: 22/100

Severity: Low

Risk: Medium

Affected:
✓ 1 Decisions
✓ 1 Code Artifacts
✓ 1 Tests
✓ 1 Governance Policies

Most Critical Dependency:
DEC-1

Governance Impact:
GOV-1
";

    assert_eq!(md, expected);
}
