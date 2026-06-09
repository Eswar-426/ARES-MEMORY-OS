use ares_orchestrator::events::replay::service::ReplayEngine;
use ares_store::db::Store;
use tempfile::tempdir;

#[tokio::test]
async fn test_replay_engine_start() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("ares.db");
    let store = Store::open(&db_path).unwrap();

    let mut conn = store.get_conn().unwrap();
    ares_store::migrations::run(&mut conn).unwrap();

    let engine = ReplayEngine::new(store);
    let job_id = engine
        .start_replay_job(Some("system.*"), None, None)
        .unwrap();

    assert!(!job_id.is_empty());
}
