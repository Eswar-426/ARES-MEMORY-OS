use ares_core::types::node::NodeType;
use ares_core::AresError;
use ares_store::Store;
use crate::models::BreakageReport;
use crate::path_engine::PathEngine;

pub struct BreakageEngine {
    path_engine: PathEngine,
}

impl BreakageEngine {
    pub fn new(store: Store) -> Self {
        Self {
            path_engine: PathEngine::new(store),
        }
    }

    /// Identifies specific engineering assets (files, tests, runtime areas) that may break due to a change.
    pub fn what_breaks(&self, target_id: &str) -> Result<BreakageReport, AresError> {
        let trace = self.path_engine.trace_downstream(target_id)?;

        let mut impacted_files = Vec::new();
        let mut impacted_tests = Vec::new();
        let mut impacted_runtime_signals = Vec::new();

        for node in &trace.nodes {
            match node.node_type {
                NodeType::File => impacted_files.push(node.label.clone()),
                NodeType::Test => impacted_tests.push(node.label.clone()),
                NodeType::RuntimeSignal => impacted_runtime_signals.push(node.label.clone()),
                _ => {}
            }
        }

        // Deterministic sorting
        impacted_files.sort();
        impacted_tests.sort();
        impacted_runtime_signals.sort();

        Ok(BreakageReport {
            impacted_files,
            impacted_tests,
            impacted_runtime_signals,
        })
    }
}
