// Services module — implemented Week 4-8
pub mod context_builder;
pub mod context_pipeline;
pub mod context_service;
pub mod contradiction_detector;
pub mod decision_intelligence;
pub mod decision_service;
pub mod graph_service;
pub mod hybrid_ranking;
pub mod memory_ranking;
pub mod memory_service;
pub mod retrieval;
pub mod scanner_service;
pub mod semantic_retrieval;

// Week 8 — Workflow orchestration
pub mod agent_registry;
pub mod execution_engine;
pub mod replay_service;
pub mod retention_manager;
pub mod workflow_analytics;
pub mod workflow_engine;
pub mod workflow_monitor;
pub mod workflow_visualizer;

#[cfg(test)]
mod context_builder_tests;
#[cfg(test)]
mod contradiction_tests;
#[cfg(test)]
mod decision_intelligence_tests;
#[cfg(test)]
mod memory_ranking_tests;
#[cfg(test)]
mod performance_tests;
#[cfg(test)]
mod retrieval_tests;

#[cfg(test)]
pub mod test_helpers {
    use ares_store::db::Store;
    use tempfile::TempDir;

    /// Create an in-memory test store with migrations applied.
    /// Returns both the store and the temp dir (keep temp dir alive!).
    pub fn test_store() -> (Store, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = dir.path().join("test.db");
        let store = Store::open(&db_path).expect("Failed to open test store");
        (store, dir)
    }
}
