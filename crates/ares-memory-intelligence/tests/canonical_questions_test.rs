use ares_memory_intelligence::facade::MemoryFacade;
use ares_requirements::impact::{ChangeRisk, ImpactSeverity, RequirementImpactReport};

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

#[test]
fn test_q18_evolution_snapshot() {
    use ares_core::RequirementId;
    use ares_requirements::{
        EventOrigin, Requirement, RequirementEvolutionEvent, RequirementEvolutionType,
        RequirementPriority, RequirementSource, RequirementStatus, RequirementTimeline,
        RequirementType,
    };

    let req = Requirement {
        id: RequirementId::from("REQ-102".to_string()),
        project_id: ares_core::ProjectId::from("proj1".to_string()),
        title: "Test".to_string(),
        description: "Desc".to_string(),
        source: RequirementSource::Product,
        requirement_type: RequirementType::Functional,
        status: RequirementStatus::Approved,
        priority: RequirementPriority::High,
        owner: None,
        created_at: 1768224000000000, // 2026-01-12
        updated_at: 1768224000000000,
        tags: vec![],
    };

    let timeline = RequirementTimeline {
        requirement_id: req.id.clone(),
        events: vec![
            RequirementEvolutionEvent {
                id: "e1".to_string(),
                requirement_id: req.id.clone(),
                timestamp: 1768224000000000,
                event_type: RequirementEvolutionType::RequirementCreated,
                event_origin: EventOrigin::Recorded,
                actor: None,
                description: "".to_string(),
                correlation_id: None,
                previous_score: None,
                new_score: None,
            },
            RequirementEvolutionEvent {
                id: "e2".to_string(),
                requirement_id: req.id.clone(),
                timestamp: 1768310400000000,
                event_type: RequirementEvolutionType::DecisionAdded,
                event_origin: EventOrigin::Recorded,
                actor: None,
                description: "DEC-1".to_string(),
                correlation_id: None,
                previous_score: None,
                new_score: None,
            },
            RequirementEvolutionEvent {
                id: "e3".to_string(),
                requirement_id: req.id.clone(),
                timestamp: 1768396800000000, // next day
                event_type: RequirementEvolutionType::CoverageImproved,
                event_origin: EventOrigin::Recorded,
                actor: None,
                description: "".to_string(),
                correlation_id: None,
                previous_score: Some(50.0),
                new_score: Some(75.0),
            },
        ],
    };

    let md = MemoryFacade::format_evolution_report(&req, &timeline);

    let expected = "\
Requirement Evolution Report

Requirement:
REQ-102

Created:
2026-01-12

Timeline:

✓ 2026-01-12 - Requirement Created
✓ 2026-01-13 - Decision Added (DEC-1)
✓ 2026-01-14 - Coverage Improved (50% → 75%)

Coverage:
75%

Drift:
None

Current Status:
Healthy
";

    assert_eq!(md, expected);
}
