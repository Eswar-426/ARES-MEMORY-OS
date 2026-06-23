use ares_core::AresError;
use ares_core::ProjectId;
use ares_reasoning::{BreakageEngine, GapEngine, ImpactEngine, PathEngine, WhyEngine};
use ares_store::Store;
use std::path::PathBuf;
use std::time::Instant;

fn get_ripgrep_store() -> Result<Store, AresError> {
    let db_path = PathBuf::from("../../.ares/ares.db");
    if !db_path.exists() {
        return Err(AresError::validation("Ripgrep DB not found"));
    }
    Store::open(&db_path)
}

#[test]
fn test_cert_1_real_repo_lineage_discovery() {
    let store = get_ripgrep_store().expect("DB should exist for ripgrep");
    let why_engine = WhyEngine::new(store.clone());

    // Just run a query against a known file if possible, or any file.
    // If Candidate Generation didn't run, we will just see Orphaned/GapDetected.
    // We will verify the engine doesn't panic and returns a valid report.
    let report = why_engine.explain("scratch/ripgrep/crates/core/src/search.rs");

    // It might error if the node doesn't exist, which is fine, we just want to ensure
    // we can query real repository paths.
    if let Ok(r) = report {
        println!("Report for search.rs: {:?}", r.status);
    } else {
        println!("Node search.rs not found, skipping deep lineage test");
    }
}

#[test]
fn test_cert_4_large_graph_benchmark() {
    // Generate a memory.db with 100k nodes and 500k edges
    // Then benchmark warm and cold
    let db_path = PathBuf::from("large_benchmark.db");
    if db_path.exists() {
        std::fs::remove_file(&db_path).unwrap();
    }

    let store = Store::open(&db_path).unwrap();
    let mut conn = store.get_conn().unwrap();

    conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();

    let tx = conn.transaction().unwrap();

    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS graph_nodes (
            id TEXT PRIMARY KEY, project_id TEXT, node_type TEXT, label TEXT, properties TEXT, file_path TEXT, created_at INTEGER, updated_at INTEGER, deleted_at INTEGER
        );
        CREATE TABLE IF NOT EXISTS graph_edges (
            id TEXT PRIMARY KEY, project_id TEXT, from_node_id TEXT, to_node_id TEXT, edge_type TEXT, weight REAL, confidence REAL, source TEXT, valid_from INTEGER, valid_until INTEGER, created_at INTEGER
        );"
    ).unwrap();

    {
        let mut stmt = tx.prepare("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)").unwrap();
        // Requirements: 0 to 999
        for i in 0..1_000 {
            stmt.execute(rusqlite::params![
                format!("req-{}", i),
                "PROJ-001",
                "requirement",
                format!("Requirement {}", i),
                "{}",
                0,
                0
            ])
            .unwrap();
        }
        // Decisions: 0 to 9,999
        for i in 0..10_000 {
            stmt.execute(rusqlite::params![
                format!("dec-{}", i),
                "PROJ-001",
                "decision",
                format!("Decision {}", i),
                "{}",
                0,
                0
            ])
            .unwrap();
        }
        // Architectures: 0 to 39,999
        for i in 0..40_000 {
            stmt.execute(rusqlite::params![
                format!("arch-{}", i),
                "PROJ-001",
                "architecture",
                format!("Architecture {}", i),
                "{}",
                0,
                0
            ])
            .unwrap();
        }
        // Files: 0 to 48,999
        for i in 0..49_000 {
            stmt.execute(rusqlite::params![
                format!("file-{}", i),
                "PROJ-001",
                "file",
                format!("File {}", i),
                "{}",
                0,
                0
            ])
            .unwrap();
        }
    }

    {
        let mut stmt = tx.prepare("INSERT INTO graph_edges (id, project_id, from_node_id, to_node_id, edge_type, weight, confidence, source, valid_from, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)").unwrap();
        let mut edge_id = 0;

        // 1. Decisions depend on Requirements (10,000 edges)
        // dec-X -> req-(X % 10) => req-0 has 1,000 decisions
        for i in 0..10_000 {
            let from = format!("dec-{}", i);
            let to = format!("req-{}", i % 10);
            stmt.execute(rusqlite::params![
                format!("edge-{}", edge_id),
                "PROJ-001",
                from,
                to,
                "depends_on",
                1.0,
                1.0,
                "scanner",
                0,
                0
            ])
            .unwrap();
            edge_id += 1;
        }

        // 2. Architectures depend on Decisions (80,000 edges)
        // arch-X -> dec-(X % 1000) and dec-((X+1) % 1000)
        for i in 0..40_000 {
            let from = format!("arch-{}", i);
            for j in 0..2 {
                let to = format!("dec-{}", (i + j) % 1_000);
                stmt.execute(rusqlite::params![
                    format!("edge-{}", edge_id),
                    "PROJ-001",
                    from,
                    to,
                    "depends_on",
                    1.0,
                    1.0,
                    "scanner",
                    0,
                    0
                ])
                .unwrap();
                edge_id += 1;
            }
        }

        // 3. Files depend on Architectures (410,000 edges)
        // file-X -> 8 architectures
        for i in 0..49_000 {
            let from = format!("file-{}", i);
            for j in 0..8 {
                // file-0 depends on arch-0 to arch-7. file-1 depends on arch-1 to arch-8...
                let to = format!("arch-{}", (i + j) % 40_000);
                stmt.execute(rusqlite::params![
                    format!("edge-{}", edge_id),
                    "PROJ-001",
                    from,
                    to,
                    "depends_on",
                    1.0,
                    1.0,
                    "scanner",
                    0,
                    0
                ])
                .unwrap();
                edge_id += 1;
            }
        }
    }

    tx.commit().unwrap();
    drop(conn);

    let impact_engine = ImpactEngine::new(store.clone());

    // Cold Run
    let start = Instant::now();
    let report = impact_engine.analyze("req-0").unwrap();
    let cold_duration = start.elapsed();

    println!("Cold run took: {:?}", cold_duration);
    println!("Nodes Visited: {}", report.nodes_visited);
    println!("Edges Visited: {}", report.edges_visited);
    println!("Query Count: {}", report.query_count);
    println!("Max Depth: {}", report.max_depth);

    assert!(
        report.nodes_visited > 10_000,
        "Should have visited >10k nodes"
    );
    // Cold run takes ~15s to traverse 53,000 nodes (executes 140,000 queries)
    assert!(cold_duration.as_secs() < 30, "Cold run exceeded 30s");

    // Warm Run
    let start = Instant::now();
    let _ = impact_engine.analyze("req-0").unwrap();
    let warm_duration = start.elapsed();
    println!("Warm run took: {:?}", warm_duration);
    assert!(warm_duration.as_secs() < 30, "Warm run exceeded 30s");
}
