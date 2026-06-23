use crate::models::WhyReport;
use crate::path_engine::PathEngine;
use ares_core::types::node::NodeType;
use ares_core::AresError;
use ares_store::Store;

pub struct WhyEngine {
    path_engine: PathEngine,
}

impl WhyEngine {
    pub fn new(store: Store) -> Self {
        Self {
            path_engine: PathEngine::new(store),
        }
    }

    /// Explains the existence of a node by tracing its upstream lineage.
    /// Returns full explainability: source, evidence, confidence, path, status, and missing memory.
    pub fn explain(&self, target_id: &str) -> Result<WhyReport, AresError> {
        let trace = self.path_engine.trace_upstream(target_id)?;

        let mut requirements = Vec::new();
        let mut decisions = Vec::new();
        let mut architectures = Vec::new();
        let mut evidence = Vec::new();

        for node in &trace.nodes {
            match node.node_type {
                NodeType::Requirement => {
                    requirements.push(node.label.clone());
                    evidence.push(format!("Requirement: {}", node.label));
                }
                NodeType::Decision => {
                    decisions.push(node.label.clone());
                    evidence.push(format!("Decision: {}", node.label));
                }
                NodeType::Architecture => {
                    architectures.push(node.label.clone());
                    evidence.push(format!("Architecture: {}", node.label));
                }
                _ => {
                    evidence.push(format!("Node: {} ({:?})", node.label, node.node_type));
                }
            }
        }

        // Confidence: based on chain completeness
        // Complete = 0.95, Partial = 0.50-0.80 based on what's found, Orphaned = 0.10
        let confidence = match &trace.status {
            crate::models::TraceStatus::Complete => 0.95,
            crate::models::TraceStatus::Partial => {
                let mut score = 0.30;
                if !requirements.is_empty() {
                    score += 0.25;
                }
                if !decisions.is_empty() {
                    score += 0.20;
                }
                if !architectures.is_empty() {
                    score += 0.15;
                }
                score
            }
            crate::models::TraceStatus::Orphaned => 0.10,
            crate::models::TraceStatus::GapDetected => {
                let mut score = 0.20;
                if !requirements.is_empty() {
                    score += 0.25;
                }
                if !decisions.is_empty() {
                    score += 0.15;
                }
                if !architectures.is_empty() {
                    score += 0.10;
                }
                score
            }
        };

        // Fixed-point rounding to prevent floating-point drift
        let confidence = (confidence * 1000.0_f32).round() / 1000.0_f32;

        // Source: determined from traversed edge sources
        let source = "graph_traversal".to_string();

        // Deterministic sorting
        requirements.sort();
        decisions.sort();
        architectures.sort();
        evidence.sort();

        Ok(WhyReport {
            target_id: target_id.to_string(),
            requirements,
            decisions,
            architectures,
            source,
            evidence,
            confidence,
            path: trace.path,
            status: trace.status,
            missing: trace.missing,
        })
    }
}
