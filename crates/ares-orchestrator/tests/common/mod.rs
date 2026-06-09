use ares_orchestrator::control::config::OrchestratorConfig;
use ares_store::db::Store;
use tempfile::TempDir;

pub fn setup_test_env() -> (Store, OrchestratorConfig, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let store = Store::open(&db_path).expect("Failed to open test store");

    let config = OrchestratorConfig::default();

    (store, config, dir)
}
