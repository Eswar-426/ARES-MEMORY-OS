use rusqlite::Connection;
use std::path::PathBuf;

#[test]
fn test_graph_fk_integrity() {
    let repo_dir = std::env::current_dir().unwrap();
    // In cargo test, current_dir is the crate root. We need to go up to the workspace root.
    let workspace_root = repo_dir.parent().unwrap().parent().unwrap();
    let db_path = workspace_root.join(".ares/ares.db");
    
    // Only run if the database exists (e.g. after a local ingest)
    if !db_path.exists() {
        return;
    }

    let conn = Connection::open(&db_path).unwrap();

    let missing_sources: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM graph_edges e LEFT JOIN graph_nodes n ON e.from_node_id = n.id WHERE n.id IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap();

    let missing_targets: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM graph_edges e LEFT JOIN graph_nodes n ON e.to_node_id = n.id WHERE n.id IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(missing_sources, 0, "Found {} edges with missing source nodes", missing_sources);
    assert_eq!(missing_targets, 0, "Found {} edges with missing target nodes", missing_targets);
}
