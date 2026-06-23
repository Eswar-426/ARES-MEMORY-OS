use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use sysinfo::{ProcessRefreshKind, System};

use serde::Serialize;

struct RepoTest {
    name: String,
    path: PathBuf,
    tier: String,
}

#[derive(Debug, Serialize)]
struct TestResult {
    name: String,
    tier: String,
    success: bool,
    time_ms: u128,
    rss_mb: f64,
    node_count: i64,
    edge_count: i64,
    gap_count: i64,
    traceability_score: f64,
}

fn main() {
    let repos = vec![
        RepoTest {
            name: "ARES".to_string(),
            path: PathBuf::from("."),
            tier: "A".to_string(),
        },
        RepoTest {
            name: "Automyra".to_string(),
            path: PathBuf::from(".temp/automyra"),
            tier: "A".to_string(),
        },
        RepoTest {
            name: "ripgrep".to_string(),
            path: PathBuf::from(".temp/ripgrep"),
            tier: "B".to_string(),
        },
        RepoTest {
            name: "cargo-watch".to_string(),
            path: PathBuf::from(".temp/cargo-watch"),
            tier: "B".to_string(),
        },
        RepoTest {
            name: "Next.js".to_string(),
            path: PathBuf::from(".temp/nextjs"),
            tier: "B".to_string(),
        },
        RepoTest {
            name: "NestJS".to_string(),
            path: PathBuf::from(".temp/nestjs"),
            tier: "B".to_string(),
        },
        RepoTest {
            name: "Turborepo".to_string(),
            path: PathBuf::from(".temp/turborepo"),
            tier: "B".to_string(),
        },
        RepoTest {
            name: "Nx Workspace".to_string(),
            path: PathBuf::from(".temp/nx"),
            tier: "B".to_string(),
        },
    ];

    let exe = std::env::current_dir()
        .unwrap()
        .join("target/release/ares.exe");
    let mut sys = System::new_with_specifics(
        sysinfo::RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );

    let mut results = Vec::new();

    for repo in repos {
        if !repo.path.exists() {
            println!("Skipping {}, path does not exist", repo.name);
            continue;
        }

        // Remove existing DB to ensure cold ingest
        let db_path = repo.path.join(".ares").join(".ares.db");
        if db_path.exists() {
            let _ = fs::remove_file(&db_path);
        }

        println!("Validating {}...", repo.name);

        let start = Instant::now();
        let mut child = Command::new(&exe)
            .arg("ingest")
            .arg(&repo.path)
            .spawn()
            .unwrap();

        let pid = sysinfo::Pid::from_u32(child.id());
        let mut peak_rss: u64 = 0;

        loop {
            if let Ok(Some(_status)) = child.try_wait() {
                break;
            }
            sys.refresh_processes();
            if let Some(proc) = sys.process(pid) {
                let rss = proc.memory();
                if rss > peak_rss {
                    peak_rss = rss;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let status = child.wait().unwrap();
        let duration = start.elapsed().as_millis();
        let rss_mb = peak_rss as f64 / (1024.0 * 1024.0);

        let mut node_count = 0;
        let mut edge_count = 0;
        let mut gap_count = 0;
        let mut fully_traceable = 0;
        let mut code_nodes = 0;

        if db_path.exists() {
            if let Ok(conn) = Connection::open(&db_path) {
                node_count = conn
                    .query_row("SELECT COUNT(*) FROM graph_entities", [], |row| row.get(0))
                    .unwrap_or(0);
                edge_count = conn
                    .query_row("SELECT COUNT(*) FROM graph_relationships", [], |row| {
                        row.get(0)
                    })
                    .unwrap_or(0);
                gap_count = conn
                    .query_row(
                        "SELECT COUNT(*) FROM graph_entities WHERE entity_type = 'KnowledgeGap'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                // Traceability approximation
                code_nodes = conn
                    .query_row(
                        "SELECT COUNT(*) FROM graph_entities WHERE entity_type = 'CodeArtifact'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                let traced: i64 = conn.query_row("SELECT COUNT(DISTINCT source_entity) FROM graph_relationships WHERE relationship_type = 'ValidatedBy'", [], |row| row.get(0)).unwrap_or(0);
                fully_traceable = traced;
            }
        }

        let mut trace_score = 0.0;
        if code_nodes > 0 {
            trace_score = (fully_traceable as f64 / code_nodes as f64) * 100.0;
        }

        let res = TestResult {
            name: repo.name.clone(),
            tier: repo.tier.clone(),
            success: status.success(),
            time_ms: duration,
            rss_mb,
            node_count,
            edge_count,
            gap_count,
            traceability_score: trace_score,
        };

        println!("{:?}", res);
        results.push(res);
    }

    // Dump results
    let json = serde_json::to_string_pretty(&results).unwrap_or_default();
    fs::write("reports/validation/real_world_results.json", json).unwrap();
}
