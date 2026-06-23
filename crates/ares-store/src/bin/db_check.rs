use ares_store::db::Store;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = env::current_dir()?.join("ares_memory.db");
    let store = Store::open(&db_path)?;
    let conn = store.get_conn()?;

    let null_nodes_count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE from_node_id IS NULL OR to_node_id IS NULL",
            (),
            |row| row.get(0),
        )
        .unwrap_or(0);

    let self_references_count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE from_node_id = to_node_id",
            (),
            |row| row.get(0),
        )
        .unwrap_or(0);

    println!("Integrity check - Null Nodes: {}", null_nodes_count);
    println!(
        "Integrity check - Self References: {}",
        self_references_count
    );

    Ok(())
}
