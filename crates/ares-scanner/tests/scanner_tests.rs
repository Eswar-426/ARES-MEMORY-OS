use ares_core::{types::event::now_micros, Project, ProjectId, ProjectMaturity};
use ares_scanner::Scanner;
use ares_store::repositories::project::SqliteProjectRepository;
use ares_store::{db::Store, repositories::graph::SqliteGraphRepository};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_full_scan_discovers_and_parses() {
    let db_dir = TempDir::new().unwrap();
    let db_path = db_dir.path().join("test.db");
    let store = Store::open(&db_path).unwrap();

    let project_repo = SqliteProjectRepository::new(store.clone());
    let graph_repo = Arc::new(SqliteGraphRepository::new(store.clone()));

    let workspace_dir = TempDir::new().unwrap();
    let code_path = workspace_dir.path().join("main.rs");
    fs::write(&code_path, "fn hello_world() {}").unwrap();

    let project_id = ProjectId::new();
    project_repo
        .create(&Project {
            id: project_id.clone(),
            name: "test_proj".into(),
            description: "".into(),
            root_path: workspace_dir.path().to_string_lossy().to_string(),
            primary_language: "rust".into(),
            domain: "".into(),
            maturity: ProjectMaturity::Greenfield,
            created_at: now_micros(),
            updated_at: now_micros(),
            deleted_at: None,
        })
        .unwrap();

    let scanner = Scanner::new(graph_repo.clone());
    scanner
        .full_scan(&project_id, workspace_dir.path())
        .unwrap();

    let nodes = graph_repo.get_by_file_path(&project_id, "main.rs").unwrap();

    assert!(!nodes.is_empty());
    let has_hello_world = nodes.iter().any(|n| n.label == "hello_world");
    assert!(
        has_hello_world,
        "Failed to find 'hello_world' in extracted nodes"
    );
}
