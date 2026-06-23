use crate::models::{ImpactReport, Reachability};
use crate::path_engine::PathEngine;
use crate::risk_engine::RiskEngine;
use ares_core::types::node::NodeType;
use ares_core::AresError;
use ares_store::Store;

pub struct ImpactEngine {
    path_engine: PathEngine,
    risk_engine: RiskEngine,
}

impl ImpactEngine {
    pub fn new(store: Store) -> Self {
        Self {
            path_engine: PathEngine::new(store),
            risk_engine: RiskEngine::new(),
        }
    }

    /// Analyzes the downstream impact of changing a target node.
    /// Classifies each affected component by Reachability (Direct/Indirect/Transitive).
    pub fn analyze(&self, target_id: &str) -> Result<ImpactReport, AresError> {
        let trace = self.path_engine.trace_downstream(target_id)?;

        let mut affected_requirements = Vec::new();
        let mut affected_decisions = Vec::new();
        let mut affected_architecture = Vec::new();
        let mut affected_files = Vec::new();
        let mut affected_tests = 0;
        let mut classifications = Vec::new();

        // Build a distance lookup from the trace
        let dist_map: std::collections::HashMap<String, usize> =
            trace.distances.iter().cloned().collect();

        for node in &trace.nodes {
            let dist = dist_map.get(&node.label).copied().unwrap_or(1);
            let reach = Reachability::from_distance(dist);
            classifications.push((node.label.clone(), reach));

            match node.node_type {
                NodeType::Requirement => affected_requirements.push(node.label.clone()),
                NodeType::Decision => affected_decisions.push(node.label.clone()),
                NodeType::Architecture => affected_architecture.push(node.label.clone()),
                NodeType::File => affected_files.push(node.label.clone()),
                NodeType::Test => affected_tests += 1,
                _ => {}
            }
        }

        let risk_score = self.risk_engine.calculate_risk_score(
            affected_requirements.len(),
            affected_decisions.len(),
            affected_architecture.len(),
            affected_files.len(),
            affected_tests,
        );

        // Fixed-point rounding to prevent floating-point drift
        let risk_score = (risk_score * 1000.0_f32).round() / 1000.0_f32;

        // Deterministic sorting
        affected_requirements.sort();
        affected_decisions.sort();
        affected_architecture.sort();
        affected_files.sort();
        classifications.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(ImpactReport {
            affected_requirements,
            affected_decisions,
            affected_architecture,
            affected_files,
            risk_score,
            classifications,
            nodes_visited: trace.nodes_visited,
            edges_visited: trace.edges_visited,
            max_depth: trace.max_depth,
            query_count: trace.query_count,
        })
    }
}
