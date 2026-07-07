use ares_core::AresError;
use std::env;

pub async fn execute_compact() -> Result<(), AresError> {
    println!("ARES Compact: Optimizing database...\n");

    let current_dir = env::current_dir().map_err(AresError::Io)?;
    let ares_dir = current_dir.join(".ares");
    let db_path = ares_dir.join("ares.db");

    if !db_path.exists() {
        println!("Database not found at {:?}. Nothing to compact.", db_path);
        return Ok(());
    }

    let initial_size = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    println!("Initial database size: {:.2} MB", initial_size as f64 / 1_048_576.0);

    let store = ares_store::db::Store::open(&db_path)?;
    let conn = store.get_conn()?;

    println!("Executing VACUUM...");
    conn.execute("VACUUM", [])
        .map_err(|e| AresError::db(format!("Failed to vacuum database: {}", e)))?;

    // We can also analyze to optimize query plans, though it's less critical for size
    println!("Executing ANALYZE...");
    let _ = conn.execute("ANALYZE", []);

    let final_size = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    println!("Final database size: {:.2} MB", final_size as f64 / 1_048_576.0);
    let saved = initial_size.saturating_sub(final_size);
    println!("Reclaimed space: {:.2} MB", saved as f64 / 1_048_576.0);
    
    println!("\nCompaction complete.");

    Ok(())
}
