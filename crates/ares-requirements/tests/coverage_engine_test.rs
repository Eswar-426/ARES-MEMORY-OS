use ares_requirements::coverage::{RequirementCoverageEngine, GraphTraceResolver, CoverageStatus};
use ares_core::RequirementId;
use ares_requirements::models::RequirementStatus;
use ares_traceability::{TraceTargetType, test_utils::TestGraphBuilder};

#[test]
fn test_full_trace_coverage() {
    let graph = TestGraphBuilder::new()
        .link_rel("REQ-1", "DEC-1", TraceTargetType::Decision, "Satisfies")
        .link_rel("DEC-1", "CODE-1", TraceTargetType::Code, "Implements")
        .link_rel("CODE-1", "TEST-1", TraceTargetType::Test, "Validates")
        .link_rel("TEST-1", "METRIC-1", TraceTargetType::RuntimeMetric, "Monitors")
        .build();

    let resolver = GraphTraceResolver::new(&graph);
    let engine = RequirementCoverageEngine::new();
    let req_id = RequirementId::from("REQ-1");
    let status = RequirementStatus::Approved;

    let coverage = engine.evaluate(&req_id, &status, true, &resolver);

    assert_eq!(coverage.status, CoverageStatus::Verified);
    assert_eq!(coverage.coverage_score, 100.0);
}

#[test]
fn test_partial_trace_coverage() {
    let graph = TestGraphBuilder::new()
        .link_rel("REQ-2", "DEC-2", TraceTargetType::Decision, "Satisfies")
        .link_rel("DEC-2", "CODE-2", TraceTargetType::Code, "Implements")
        // Missing Test and Metric
        .build();

    let resolver = GraphTraceResolver::new(&graph);
    let engine = RequirementCoverageEngine::new();
    let req_id = RequirementId::from("REQ-2");
    let status = RequirementStatus::Approved;

    let coverage = engine.evaluate(&req_id, &status, true, &resolver);

    assert_eq!(coverage.status, CoverageStatus::Partial);
    assert!(coverage.coverage_score < 100.0 && coverage.coverage_score > 0.0);
    
    // Also verify gaps
    let gap_types: Vec<_> = coverage.gaps.iter().map(|g| g.gap_type.clone()).collect();
    assert!(gap_types.contains(&ares_requirements::coverage::GapType::MissingTests));
    assert!(gap_types.contains(&ares_requirements::coverage::GapType::MissingRuntimeMetric));
}

#[test]
fn test_orphan_requirement() {
    let graph = TestGraphBuilder::new().build(); // Empty graph downstream for REQ-3

    let resolver = GraphTraceResolver::new(&graph);
    let engine = RequirementCoverageEngine::new();
    let req_id = RequirementId::from("REQ-3");
    let status = RequirementStatus::Approved;

    let coverage = engine.evaluate(&req_id, &status, true, &resolver);

    assert_eq!(coverage.status, CoverageStatus::Orphaned);
    assert_eq!(coverage.coverage_score, 0.0);
    
    let gap_types: Vec<_> = coverage.gaps.iter().map(|g| g.gap_type.clone()).collect();
    assert!(gap_types.contains(&ares_requirements::coverage::GapType::MissingDecision));
}
