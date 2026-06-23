use crate::models::{
    DriftConfidence, DriftEvidence, DriftSeverity, RequirementBaseline, RequirementDriftReport,
    RequirementDriftType, SemanticDrift, StructuralDrift,
};
use crate::trace_analysis::TraceAnalysisEngine;
use ares_traceability::{TraceTargetType, TraceabilityGraph};

pub struct RequirementDriftEngine<'a> {
    graph: &'a TraceabilityGraph,
}

impl<'a> RequirementDriftEngine<'a> {
    pub fn new(graph: &'a TraceabilityGraph) -> Self {
        Self { graph }
    }

    pub fn evaluate_drift(&self, baseline: &RequirementBaseline) -> Option<RequirementDriftReport> {
        let resolver = TraceAnalysisEngine::new(self.graph);
        let req_id = &baseline.requirement_id;

        let current_decisions = resolver.get_downstream(req_id, TraceTargetType::Decision);
        let current_impls = resolver.get_downstream(req_id, TraceTargetType::Code);
        let current_tests = resolver.get_downstream(req_id, TraceTargetType::Test);
        let current_metrics = resolver.get_downstream(req_id, TraceTargetType::RuntimeMetric);

        let mut drift_types = Vec::new();
        let mut evidence = Vec::new();
        let mut explanations = Vec::new();
        let mut remediations = Vec::new();

        // Check for structural drift (missing elements compared to expected graph structure)
        if current_decisions.is_empty() && !baseline.decision_ids.is_empty() {
            drift_types.push(RequirementDriftType::Structural(
                StructuralDrift::MissingDecision,
            ));
            evidence.push(DriftEvidence {
                source_node: req_id.clone(),
                target_node: "Decision".to_string(),
                relationship: "ApprovedBy".to_string(),
                observed_state: "0 decisions".to_string(),
                expected_state: format!("{} decisions", baseline.decision_ids.len()),
            });
            explanations.push(
                "The requirement is no longer tied to any architectural decision.".to_string(),
            );
            remediations.push(
                "Re-link the requirement to an active Architecture Decision Record (ADR)."
                    .to_string(),
            );
        }

        if current_impls.is_empty() && !baseline.implementation_ids.is_empty() {
            drift_types.push(RequirementDriftType::Structural(
                StructuralDrift::MissingImplementation,
            ));
            evidence.push(DriftEvidence {
                source_node: req_id.clone(),
                target_node: "Code".to_string(),
                relationship: "ImplementedBy".to_string(),
                observed_state: "0 implementations".to_string(),
                expected_state: format!("{} implementations", baseline.implementation_ids.len()),
            });
            explanations
                .push("The requirement has lost its traceability to the codebase.".to_string());
            remediations.push(
                "Ensure code blocks are tagged with the requirement ID or decision ID.".to_string(),
            );
        }

        if current_tests.is_empty() && !baseline.test_ids.is_empty() {
            drift_types.push(RequirementDriftType::Structural(
                StructuralDrift::MissingVerification,
            ));
            evidence.push(DriftEvidence {
                source_node: req_id.clone(),
                target_node: "Test".to_string(),
                relationship: "VerifiedBy".to_string(),
                observed_state: "0 tests".to_string(),
                expected_state: format!("{} tests", baseline.test_ids.len()),
            });
            explanations.push(
                "Verification tests for this requirement have been removed or disconnected."
                    .to_string(),
            );
            remediations.push(
                "Add tests that explicitly verify this requirement's acceptance criteria."
                    .to_string(),
            );
        }

        if current_metrics.is_empty() && !baseline.runtime_metrics.is_empty() {
            drift_types.push(RequirementDriftType::Structural(
                StructuralDrift::MissingMonitoring,
            ));
            evidence.push(DriftEvidence {
                source_node: req_id.clone(),
                target_node: "RuntimeMetric".to_string(),
                relationship: "MonitoredBy".to_string(),
                observed_state: "0 metrics".to_string(),
                expected_state: format!("{} metrics", baseline.runtime_metrics.len()),
            });
            explanations
                .push("Runtime monitoring for this requirement is no longer active.".to_string());
            remediations.push(
                "Restore observability metrics tracking this requirement's SLAs.".to_string(),
            );
        }

        // Check for semantic drift (changes in linked elements)
        let decision_changed = baseline
            .decision_ids
            .iter()
            .any(|id| !current_decisions.contains(id));
        if decision_changed {
            drift_types.push(RequirementDriftType::Semantic(
                SemanticDrift::DecisionChanged,
            ));
            evidence.push(DriftEvidence {
                source_node: req_id.clone(),
                target_node: "Decision".to_string(),
                relationship: "ApprovedBy".to_string(),
                observed_state: "Altered decision set".to_string(),
                expected_state: "Original approved decisions".to_string(),
            });
            explanations.push("The underlying architectural decisions supporting this requirement have changed since approval.".to_string());
            remediations.push(
                "Review new decisions to ensure they still satisfy the requirement intent."
                    .to_string(),
            );
        }

        let impl_changed = baseline
            .implementation_ids
            .iter()
            .any(|id| !current_impls.contains(id));
        if impl_changed {
            drift_types.push(RequirementDriftType::Semantic(
                SemanticDrift::ImplementationChanged,
            ));
            evidence.push(DriftEvidence {
                source_node: req_id.clone(),
                target_node: "Code".to_string(),
                relationship: "ImplementedBy".to_string(),
                observed_state: "Altered implementation set".to_string(),
                expected_state: "Original implementation files".to_string(),
            });
            explanations.push(
                "The codebase files originally implementing this requirement have diverged."
                    .to_string(),
            );
            remediations.push(
                "Verify that the new code implementations still cover the required functionality."
                    .to_string(),
            );
        }

        if drift_types.is_empty() {
            return None;
        }

        let severity = if drift_types.iter().any(|d| {
            matches!(
                d,
                RequirementDriftType::Semantic(SemanticDrift::RuntimeMismatch)
            ) || matches!(
                d,
                RequirementDriftType::Structural(StructuralDrift::MissingDecision)
            )
        }) {
            DriftSeverity::Critical
        } else if drift_types.iter().any(|d| {
            matches!(
                d,
                RequirementDriftType::Structural(StructuralDrift::MissingVerification)
            ) || matches!(
                d,
                RequirementDriftType::Structural(StructuralDrift::MissingImplementation)
            ) || matches!(
                d,
                RequirementDriftType::Semantic(SemanticDrift::RequirementExpired)
            )
        }) {
            DriftSeverity::High
        } else if drift_types.iter().any(|d| {
            matches!(
                d,
                RequirementDriftType::Structural(StructuralDrift::MissingMonitoring)
            )
        }) {
            DriftSeverity::Medium
        } else {
            DriftSeverity::Low
        };

        // For now, default confidence based on severity or structural vs semantic
        let confidence = if drift_types
            .iter()
            .any(|d| matches!(d, RequirementDriftType::Structural(_)))
        {
            DriftConfidence::Certain
        } else {
            DriftConfidence::Medium
        };

        Some(RequirementDriftReport {
            requirement_id: req_id.clone(),
            severity,
            drift_types,
            evidence,
            confidence,
            explanations,
            remediations,
        })
    }
}
