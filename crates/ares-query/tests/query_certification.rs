use ares_core::ProjectId;
use ares_query::services::{
    BootstrapCandidateQueryService, BootstrapCoverageQueryService, BootstrapGapClosureQueryService,
    LifecycleDecayQueryService, LifecycleRevalidationQueryService, LifecycleStatusQueryService,
    LifecycleTrustQueryService, RepositoryValidationQueryService,
};

#[test]
fn test_bootstrap_candidate_query_service_contract() {
    let project_id = ProjectId::new();
    let result = BootstrapCandidateQueryService::execute(&project_id);

    // Verify DTO structure
    assert!(result.metadata.confidence >= 0.0);
    assert!(!result.evidence.node_ids.is_empty());
    // Candidate service is non-deterministic (heuristic) but we still verify structure
    assert!(result.data.candidates_inferred > 0);
}

#[test]
fn test_bootstrap_coverage_query_service_contract() {
    let project_id = ProjectId::new();
    let result = BootstrapCoverageQueryService::execute(&project_id);

    assert!(result.metadata.confidence > 0.0);
    assert!(!result.evidence.node_ids.is_empty());
    assert!(result.data.architectural_coverage > 0.0);
}

#[test]
fn test_bootstrap_gap_closure_query_service_contract() {
    let project_id = ProjectId::new();
    let result = BootstrapGapClosureQueryService::execute(&project_id);

    assert!(result.metadata.confidence > 0.0);
    assert!(!result.evidence.node_ids.is_empty());
}

#[test]
fn test_lifecycle_status_query_service_contract() {
    let project_id = ProjectId::new();
    let result = LifecycleStatusQueryService::execute(&project_id, "node-1");

    assert!(result.metadata.confidence > 0.0);
    assert!(!result.evidence.node_ids.is_empty());
}

#[test]
fn test_lifecycle_trust_query_service_contract() {
    let project_id = ProjectId::new();
    let result = LifecycleTrustQueryService::execute(&project_id, "node-1");

    assert!(result.metadata.confidence > 0.0);
    assert!(!result.evidence.node_ids.is_empty());
    assert!(result.data.trust_score > 0.0);
}

#[test]
fn test_lifecycle_decay_query_service_contract() {
    let project_id = ProjectId::new();
    let result = LifecycleDecayQueryService::execute(&project_id, "node-1");

    assert!(result.metadata.confidence > 0.0);
    assert!(!result.evidence.node_ids.is_empty());
    assert!(result.data.decay_rate >= 0.0);
}

#[test]
fn test_lifecycle_revalidation_query_service_contract() {
    let project_id = ProjectId::new();
    let result = LifecycleRevalidationQueryService::execute(&project_id, "node-1");

    assert!(result.metadata.confidence > 0.0);
    assert!(!result.evidence.node_ids.is_empty());
}

#[test]
fn test_repository_validation_query_service_contract() {
    let project_id = ProjectId::new();
    let result = RepositoryValidationQueryService::execute(&project_id);

    assert!(result.metadata.confidence > 0.0);
    assert!(!result.evidence.node_ids.is_empty());
}
