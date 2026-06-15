use ares_core::ProjectId;
use ares_scanner::scanner::Scanner;
use ares_store::db::Store;
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_id = ProjectId::new();
    let root_path = std::env::current_dir()?;
    println!("Scanning repository at: {}", root_path.display());

    let dir = std::env::temp_dir().join(format!("ares_test_{}", project_id.as_str()));
    let db_path = dir.join("test.db");
    let store = Store::open(&db_path)?;
    
    // Insert the project to satisfy foreign key constraints
    let conn = store.get_conn()?;
    let insert_query = format!("INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('{}', 'ARES_Memory_os', '', '{}', 'rust', '', 'mature', 0, 0)", project_id.as_str(), root_path.to_string_lossy().replace("\\", "/"));
    conn.execute(&insert_query, ())?;

    let graph_repo = Arc::new(SqliteGraphRepository::new(store.clone()));

    let scanner = Scanner::new(graph_repo.clone());
    
    // Run full scan
    scanner.full_scan(&project_id, &root_path)?;

    println!("Scan Complete!");

    Ok(())
}
