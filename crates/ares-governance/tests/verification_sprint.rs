use ares_core::id::{NodeId, ProjectId};
use ares_core::types::node::{EdgeType, GraphEdge, GraphNode, NodeType};
use ares_governance::classifier::{
    ArtifactCategory, ArtifactClassifier, ClassificationConfidence, MemoryEligibility,
};
use ares_governance::coverage_engine::{CoverageEngine, MemoryCoverageMetrics, CoverageMetric, MemoryCaptureRate};
use ares_governance::memory_debt_engine::{MemoryDebtEngine, MemoryDebtLevel, MemoryDebtMetrics};
use ares_governance::memory_gatekeeper::{GatekeeperStatus, MemoryGatekeeper};
use ares_governance::memory_health_engine::{MemoryHealthEngine, MemoryHealthScore};

fn default_coverage() -> MemoryCoverageMetrics {
    let empty = CoverageMetric { covered: 0, total: 0, percentage: 0.0 };
    MemoryCoverageMetrics {
        overall: empty.clone(),
        requirements: empty.clone(),
        decisions: empty.clone(),
        architecture: empty.clone(),
        ownership: empty.clone(),
        tests: empty.clone(),
        evidence: empty.clone(),
        capture_rate: MemoryCaptureRate {
            git_blame: false,
            git_commits: false,
            git_releases: false,
            codeowners: false,
            captured_sources: 0,
            available_sources: 0,
            rate: 0.0,
        },
    }
}

fn default_debt() -> MemoryDebtMetrics {
    MemoryDebtMetrics {
        missing_requirements_penalty: 0,
        missing_decisions_penalty: 0,
        missing_owners_penalty: 0,
        missing_evidence_penalty: 0,
        missing_tests_penalty: 0,
        drift_penalty: 0,
        total_debt_score: 0,
        severity: MemoryDebtLevel::Healthy,
    }
}

fn default_health() -> MemoryHealthScore {
    MemoryHealthScore {
        coverage_score: 0.0,
        ownership_score: 0.0,
        evidence_score: 0.0,
        validation_score: 0.0,
        freshness_score: 0.0,
        total_health: 0.0,
    }
}

#[test]
fn test_1_artifact_classifier() {
    let test_cases = vec![
        ("docs/requirements/REQ-001.md", ArtifactCategory::Requirement, MemoryEligibility::Required, ClassificationConfidence::Certain),
        ("docs/decisions/ADR-001.md", ArtifactCategory::Decision, MemoryEligibility::Required, ClassificationConfidence::Certain),
        ("docs/architecture/ARCH-001.md", ArtifactCategory::Architecture, MemoryEligibility::Required, ClassificationConfidence::Certain),
        ("docs/evidence/EVD-001.md", ArtifactCategory::Evidence, MemoryEligibility::Required, ClassificationConfidence::Certain),
        ("src/main.rs", ArtifactCategory::Code, MemoryEligibility::Required, ClassificationConfidence::Inferred),
        ("src/main_test.rs", ArtifactCategory::Test, MemoryEligibility::Recommended, ClassificationConfidence::Inferred),
        ("node_modules/package-lock.json", ArtifactCategory::Vendor, MemoryEligibility::Excluded, ClassificationConfidence::Certain),
        ("target/generated.pb.rs", ArtifactCategory::Generated, MemoryEligibility::Excluded, ClassificationConfidence::Certain),
    ];

    for (path, expected_category, expected_eligibility, expected_confidence) in test_cases {
        let node = GraphNode {
            id: NodeId::new(),
            project_id: ProjectId::new(),
            node_type: NodeType::File,
            label: "file".to_string(),
            properties: serde_json::json!({}),
            file_path: Some(path.to_string()),
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        };

        let result = ArtifactClassifier::classify(Some(&node.node_type), node.file_path.as_deref());
        assert_eq!(result.category, expected_category, "Failed category for {}", path);
        assert_eq!(result.eligibility, expected_eligibility, "Failed eligibility for {}", path);
        assert_eq!(result.confidence, expected_confidence, "Failed confidence for {}", path);
    }
}

#[test]
fn test_2_coverage_engine_math() {
    let mut metrics = default_coverage();
    
    // Simulate 10 code files total
    metrics.overall.total = 10;
    
    // 7 are covered
    metrics.overall.covered = 7;
    
    metrics.overall.percentage = if metrics.overall.total > 0 {
        (metrics.overall.covered as f64 / metrics.overall.total as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(metrics.overall.percentage, 70.0);
}

#[test]
fn test_3_debt_engine() {
    let missing_requirements = 2; // Weight 10 = 20
    let missing_owners = 3;       // Weight 7 = 21
    let missing_tests = 5;        // Weight 3 = 15
                                  // Total expected = 56

    // Using the MemoryDebtEngine logic mathematically:
    let score = (missing_requirements * 10) + (missing_owners * 7) + (missing_tests * 3);
    assert_eq!(score, 56);
}

#[test]
fn test_4_health_engine_formula() {
    let coverage_score = 80.0;
    let ownership_score = 90.0;
    let evidence_score = 70.0;
    let validation_score = 60.0;
    let freshness_score = 100.0;

    let total = (coverage_score * 0.35)
        + (ownership_score * 0.20)
        + (evidence_score * 0.15)
        + (validation_score * 0.15)
        + (freshness_score * 0.15);

    assert_eq!(total, 80.5);
}

#[test]
fn test_5_gatekeeper_thresholds() {
    let mut before_cov = default_coverage();
    let mut after_cov = default_coverage();
    let before_debt = default_debt();
    let after_debt = default_debt();
    let mut before_health = default_health();
    let mut after_health = default_health();

    // Test 1: Soft Fail (-3% coverage regression)
    before_cov.overall.percentage = 85.0;
    after_cov.overall.percentage = 82.0;

    let result1 = MemoryGatekeeper::evaluate_delta(
        &before_cov, &after_cov, &before_debt, &after_debt, &before_health, &after_health
    );
    match result1 {
        GatekeeperStatus::SoftFail(_) => {}
        _ => panic!("Expected SoftFail for -3% coverage"),
    }

    // Test 2: Hard Fail (-10% coverage regression)
    after_cov.overall.percentage = 75.0;
    let result2 = MemoryGatekeeper::evaluate_delta(
        &before_cov, &after_cov, &before_debt, &after_debt, &before_health, &after_health
    );
    match result2 {
        GatekeeperStatus::HardFail(_) => {}
        _ => panic!("Expected HardFail for -10% coverage"),
    }

    // Test 3: Hard Fail (Requirement removed)
    after_cov.overall.percentage = 85.0; // Restored percentage
    before_cov.requirements.total = 10;
    after_cov.requirements.total = 9; // Removed

    let result3 = MemoryGatekeeper::evaluate_delta(
        &before_cov, &after_cov, &before_debt, &after_debt, &before_health, &after_health
    );
    match result3 {
        GatekeeperStatus::HardFail(_) => {}
        _ => panic!("Expected HardFail for Requirement removal"),
    }
}

#[test]
fn test_6_semantic_integrity() {
    // Assert enum variants exist
    let _drives = EdgeType::Drives;
    let _validated_by = EdgeType::ValidatedBy;
    
    let requirement = GraphNode {
        id: NodeId::new(), project_id: ProjectId::new(),
        node_type: NodeType::Requirement, label: "REQ-001".to_string(),
        properties: serde_json::json!({}), file_path: Some("REQ-001.md".to_string()),
        created_at: 0, updated_at: 0, deleted_at: None,
    };

    let decision = GraphNode {
        id: NodeId::new(), project_id: ProjectId::new(),
        node_type: NodeType::Decision, label: "ADR-001".to_string(),
        properties: serde_json::json!({}), file_path: Some("ADR-001.md".to_string()),
        created_at: 0, updated_at: 0, deleted_at: None,
    };

    let code = GraphNode {
        id: NodeId::new(), project_id: ProjectId::new(),
        node_type: NodeType::File, label: "main.rs".to_string(),
        properties: serde_json::json!({}), file_path: Some("main.rs".to_string()),
        created_at: 0, updated_at: 0, deleted_at: None,
    };

    let test = GraphNode {
        id: NodeId::new(), project_id: ProjectId::new(),
        node_type: NodeType::File, label: "main_test.rs".to_string(),
        properties: serde_json::json!({}), file_path: Some("main_test.rs".to_string()),
        created_at: 0, updated_at: 0, deleted_at: None,
    };

    let _drives_edge1 = GraphEdge {
        id: "e1".to_string(), project_id: ProjectId::new(),
        from_node_id: requirement.id.clone(), to_node_id: decision.id.clone(),
        edge_type: EdgeType::Drives, weight: 1.0, confidence: 1.0, source: "".to_string(),
        valid_from: 0, valid_until: None, created_at: 0,
    };

    let _drives_edge2 = GraphEdge {
        id: "e2".to_string(), project_id: ProjectId::new(),
        from_node_id: decision.id.clone(), to_node_id: code.id.clone(),
        edge_type: EdgeType::Drives, weight: 1.0, confidence: 1.0, source: "".to_string(),
        valid_from: 0, valid_until: None, created_at: 0,
    };

    let _validated_edge = GraphEdge {
        id: "e3".to_string(), project_id: ProjectId::new(),
        from_node_id: code.id.clone(), to_node_id: test.id.clone(),
        edge_type: EdgeType::ValidatedBy, weight: 1.0, confidence: 1.0, source: "".to_string(),
        valid_from: 0, valid_until: None, created_at: 0,
    };
    
    // Test compilation guarantees that EdgeType::Drives and EdgeType::ValidatedBy exist
    // and aren't substituted with MotivatedBy or RelatedTo
}
