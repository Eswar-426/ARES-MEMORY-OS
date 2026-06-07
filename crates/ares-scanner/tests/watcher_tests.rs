use ares_core::ProjectId;
use ares_scanner::watcher::ProjectWatcher;

use std::fs;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_watcher_detects_changes() {
    let dir = TempDir::new().unwrap();
    let project_id = ProjectId::new();
    let watcher = ProjectWatcher::new(project_id, dir.path().to_path_buf());

    let rx = watcher.watch().unwrap();

    // Give the background watcher thread time to initialize
    std::thread::sleep(Duration::from_millis(500));

    // Create a new file
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, "fn test() {}").unwrap();

    // Wait for debounce
    let result = rx.recv_timeout(Duration::from_secs(3));
    assert!(
        result.is_ok(),
        "Watcher failed to detect change within timeout"
    );
    let paths = result.unwrap();
    assert!(!paths.is_empty());

    // On Windows, paths might be prefixed with \\?\, so check ends_with
    assert!(
        paths.iter().any(|p| p.ends_with("test.rs")),
        "Expected paths to contain test.rs, found: {:?}",
        paths
    );
}
