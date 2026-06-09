use ares_orchestrator::events::consumer_groups::service::ConsumerGroupService;
use ares_store::db::Store;
use tempfile::tempdir;

#[tokio::test]
async fn test_consumer_group_offset_commit() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("ares.db");
    let store = Store::open(&db_path).unwrap();

    let mut conn = store.get_conn().unwrap();
    ares_store::migrations::run(&mut conn).unwrap();

    // First insert a consumer group to satisfy foreign key constraints
    conn.execute(
        "INSERT INTO event_consumer_groups (id, name, topic_pattern, status, created_at, updated_at)
         VALUES ('cg_1', 'Test Group', 'system.*', 'active', 0, 0)",
        [],
    ).unwrap();

    let service = ConsumerGroupService::new(store);
    let result = service.commit_offset("cg_1", "part_0", "evt_123");

    assert!(result.is_ok(), "Failed to commit offset: {:?}", result);
}
