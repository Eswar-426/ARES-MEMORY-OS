use ares_requirements::impact::RequirementImpactEngine;
use ares_traceability::{test_utils::TestGraphBuilder, TraceTargetType};

/// A rudimentary simulator stub that anticipates Phase 9F
fn simulate_change(req_id: &str, graph: &ares_traceability::TraceabilityGraph) -> Vec<String> {
    // Phase 9F will perform deep contextual simulation.
    // For consistency, it must touch all downstream dependencies in the trace graph.
    let mut affected = Vec::new();
    if let Ok(nodes) = graph.find_downstream(req_id) {
        for n in nodes {
            affected.push(n.id.clone());
        }
    }
    affected
}

#[test]
fn test_impact_vs_simulation_parity() {
    let graph = TestGraphBuilder::new()
        .link_rel("REQ-1", "DEC-1", TraceTargetType::Decision, "Satisfies")
        .link_rel("DEC-1", "CODE-1", TraceTargetType::Code, "Implements")
        .build();

    let engine = RequirementImpactEngine::new(&graph);
    let impact_report = engine.evaluate_impact("REQ-1");

    let total_impact_artifacts = impact_report.affected_decisions.len()
        + impact_report.affected_code.len()
        + impact_report.affected_tests.len()
        + impact_report.affected_runtime_metrics.len()
        + impact_report.affected_governance.len()
        + impact_report.affected_architecture.len();

    let simulated_artifacts = simulate_change("REQ-1", &graph);

    // Impact Engine says X artifacts, Simulator says Y artifacts. They must match!
    assert_eq!(
        total_impact_artifacts,
        simulated_artifacts.len(),
        "Impact Engine and Simulator parity mismatch"
    );
}
