use crate::{ContextSelector, TokenBudget};
use ares_core::{id::new_id, ProjectId};
use ares_store::Store;
use tempfile::TempDir;

#[tokio::test]
async fn test_empty_context() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let store = Store::open(&db_path).expect("Failed to open test store");
    let selector = ContextSelector::new(store);
    let project_id = ProjectId::from(new_id()).to_string();

    let package = selector
        .build_package(&project_id, "test prompt", TokenBudget::Medium)
        .await
        .unwrap();

    assert_eq!(package.original_prompt, "test prompt");
    assert_eq!(package.architecture_nodes.len(), 0);
    assert_eq!(package.decisions.len(), 0);
    assert_eq!(package.bugs.len(), 0);
    assert_eq!(package.memories.len(), 0);
}
