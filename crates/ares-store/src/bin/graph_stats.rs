use ares_store::db::Store;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_path = std::env::current_dir()?;
    let db_path = root_path.join("ares_memory.db");
    let store = Store::open(&db_path)?;

    println!("Calculating graph metrics...");
    let metrics = store.graph_metrics()?;
    let call_metrics = store.call_graph_metrics()?;

    let val_dir = root_path.join("artifacts").join("validation");
    if !val_dir.exists() {
        fs::create_dir_all(&val_dir)?;
    }

    let stats_file = val_dir.join("graph_stats.json");
    fs::write(&stats_file, serde_json::to_string_pretty(&metrics)?)?;

    let call_stats_file = val_dir.join("call_graph_metrics.json");
    fs::write(
        &call_stats_file,
        serde_json::to_string_pretty(&call_metrics)?,
    )?;

    // Graph Evolution Tracking
    let history_dir = root_path
        .join("artifacts")
        .join("history")
        .join("graph_metrics");
    if !history_dir.exists() {
        fs::create_dir_all(&history_dir)?;
    }

    let now = chrono::Utc::now().to_rfc3339();
    let snapshot = ares_store::metrics::GraphEvolutionSnapshot {
        timestamp: now.clone(),
        nodes: metrics.total_nodes,
        edges: metrics.total_edges,
        largest_component: metrics.largest_connected_component,
    };

    // Save timestamped snapshot
    let timestamp_str = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let snapshot_file = history_dir.join(format!("snapshot_{}.json", timestamp_str));
    fs::write(&snapshot_file, serde_json::to_string_pretty(&snapshot)?)?;

    println!("Graph stats saved to {}", stats_file.display());
    println!("Call graph metrics saved to {}", call_stats_file.display());
    println!(
        "Graph evolution snapshot saved to {}",
        snapshot_file.display()
    );
    Ok(())
}
