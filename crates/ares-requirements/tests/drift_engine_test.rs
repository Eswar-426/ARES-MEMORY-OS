use ares_requirements::drift::RequirementDriftEngine;
use ares_requirements::models::{DriftSeverity, RequirementDriftType, StructuralDrift};
use ares_requirements::RequirementBaseline;
use ares_traceability::{test_utils::TestGraphBuilder, TraceTargetType};

#[test]
fn test_structural_drift_missing_decision() {
    let graph = TestGraphBuilder::new()
        .link_rel("DEC-1", "CODE-1", TraceTargetType::Code, "Implements")
        .build();

    let engine = RequirementDriftEngine::new(&graph);

    let baseline = RequirementBaseline {
        requirement_id: "REQ-1".to_string(),
        approved_at: 0,
        decision_ids: vec!["DEC-1".to_string()],
        implementation_ids: vec![],
        test_ids: vec![],
        runtime_metrics: vec![],
    };

    let report_opt = engine.evaluate_drift(&baseline);
    assert!(report_opt.is_some());
    let report = report_opt.unwrap();

    assert_eq!(report.requirement_id, "REQ-1");
    assert!(report.drift_types.len() >= 1);

    assert!(report
        .drift_types
        .contains(&RequirementDriftType::Structural(
            StructuralDrift::MissingDecision
        )));
    assert_eq!(report.severity, DriftSeverity::Critical);
}

#[test]
fn test_structural_drift_missing_code() {
    let graph = TestGraphBuilder::new()
        .link_rel("REQ-2", "DEC-2", TraceTargetType::Decision, "Satisfies")
        .build();

    let engine = RequirementDriftEngine::new(&graph);

    let baseline = RequirementBaseline {
        requirement_id: "REQ-2".to_string(),
        approved_at: 0,
        decision_ids: vec!["DEC-2".to_string()],
        implementation_ids: vec!["CODE-2".to_string()],
        test_ids: vec![],
        runtime_metrics: vec![],
    };

    let report_opt = engine.evaluate_drift(&baseline);
    assert!(report_opt.is_some());
    let report = report_opt.unwrap();

    assert_eq!(report.requirement_id, "REQ-2");
    assert!(report.drift_types.len() >= 1);

    assert!(report
        .drift_types
        .contains(&RequirementDriftType::Structural(
            StructuralDrift::MissingImplementation
        )));
    assert_eq!(report.severity, DriftSeverity::High);
}
