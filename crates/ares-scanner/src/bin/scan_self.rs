use ares_core::ProjectId;
use ares_scanner::scanner::Scanner;
use ares_store::db::Store;
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_id = ProjectId::new();
    let root_path = std::env::current_dir()?;
    println!("Scanning repository at: {}", root_path.display());

    let db_path = root_path.join("ares_memory.db");
    let store = Store::open(&db_path)?;

    let conn = store.get_conn()?;
    let insert_query = format!("INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('{}', 'ARES_Memory_os', '', '{}', 'rust', '', 'mature', 0, 0)", project_id.as_str(), root_path.to_string_lossy().replace("\\", "/"));
    conn.execute(&insert_query, ())?;

    let existing_project_id: String = conn.query_row(
        &format!(
            "SELECT id FROM projects WHERE root_path = '{}'",
            root_path.to_string_lossy().replace("\\", "/")
        ),
        (),
        |row| row.get(0),
    )?;
    let project_id = ares_core::ProjectId::from(existing_project_id);

    let graph_repo = Arc::new(SqliteGraphRepository::new(store.clone()));

    let scanner = Scanner::new(graph_repo.clone());

    // Run full scan
    let report = scanner.full_scan(&project_id, &root_path)?;

    println!("Scan Complete!");

    let out_dir = root_path.join("artifacts").join("validation");
    if !out_dir.exists() {
        std::fs::create_dir_all(&out_dir)?;
    }

    let out_file = out_dir.join("scanner_report.json");
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&out_file, json)?;

    println!("Scanner report saved to {}", out_file.display());

    Ok(())
}
