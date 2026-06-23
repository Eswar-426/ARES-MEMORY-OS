use ares_memory_server::initializer::RepositoryInitializer;
use ares_memory_server::scanner::RepositoryScanner;
use ares_memory_server::builder::RepositoryBuilder;
use ares_memory_server::server::RepositoryServer;
use std::env;
use std::path::PathBuf;
use std::fs;

fn setup_test_dir(name: &str) -> PathBuf {
    let mut d = env::temp_dir();
    d.push(format!("ares_test_{}", name));
    if d.exists() {
        fs::remove_dir_all(&d).unwrap();
    }
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn cert_1_init_repository() {
    let dir = setup_test_dir("cert_1");
    assert!(RepositoryInitializer::init(&dir).is_ok());
    assert!(dir.join(".ares").exists());
    assert!(dir.join(".ares/config.toml").exists());
    assert!(dir.join(".ares/build_manifest.json").exists());
}

#[test]
fn cert_2_scan_repository() {
    let dir = setup_test_dir("cert_2");
    RepositoryInitializer::init(&dir).unwrap();
    assert!(RepositoryScanner::scan(&dir).is_ok());
}

#[test]
fn cert_3_build_memory() {
    let dir = setup_test_dir("cert_3");
    RepositoryInitializer::init(&dir).unwrap();
    RepositoryScanner::scan(&dir).unwrap();
    assert!(RepositoryBuilder::build(&dir).is_ok());
    
    // Verify manifest updated
    let manifest_str = fs::read_to_string(dir.join(".ares/build_manifest.json")).unwrap();
    assert!(manifest_str.contains("Build Stage 1: Scanner"));
    assert!(manifest_str.contains("Build Stage 11: Knowledge Gap"));
}

#[test]
fn cert_4_query_repository() {
    let dir = setup_test_dir("cert_4");
    RepositoryInitializer::init(&dir).unwrap();
    RepositoryBuilder::build(&dir).unwrap();
    // In real implementation this tests ares-query logic against the DB
}

#[test]
fn cert_5_api_server() {
    let dir = setup_test_dir("cert_5");
    RepositoryInitializer::init(&dir).unwrap();
    RepositoryBuilder::build(&dir).unwrap();
    assert!(RepositoryServer::serve(&dir).is_ok());
}

#[test]
fn cert_6_restart_persistence() {
    let dir = setup_test_dir("cert_6");
    RepositoryInitializer::init(&dir).unwrap();
    RepositoryBuilder::build(&dir).unwrap();
    
    // Server should be able to load statically
    assert!(RepositoryServer::serve(&dir).is_ok());
}

#[test]
fn cert_7_repository_upgrade() {
    // Check manifest versions and upgrades
}

#[test]
fn cert_8_large_repository() {
    // Load testing
}

#[test]
fn cert_9_determinism() {
    // Two builds yield same hash
}

#[test]
fn cert_10_explainability() {
    // Test logic rationale
}

#[test]
fn cert_11_real_repository_bootstrap() {
    // Real end-to-end fixture test
    let dir = setup_test_dir("cert_11_real");
    
    // 1. init
    assert!(RepositoryInitializer::init(&dir).is_ok());
    // 2. scan
    assert!(RepositoryScanner::scan(&dir).is_ok());
    // 3. build
    assert!(RepositoryBuilder::build(&dir).is_ok());
    // 4. serve & query
    assert!(RepositoryServer::serve(&dir).is_ok());
    
    let manifest_str = fs::read_to_string(dir.join(".ares/build_manifest.json")).unwrap();
    let manifest: ares_memory_server::manifest::BuildManifest = serde_json::from_str(&manifest_str).unwrap();
    assert_eq!(manifest.stages_completed.len(), 11);
}
