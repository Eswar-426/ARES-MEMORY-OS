use ares_context::ContextEngine;
use ares_context::traversal::TraversalConfig;
use ares_core::ProjectId;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;
use ares_context::pack::{ContextPackBuilder, ContextPackValidator};
use ares_context::models::pack::ContextBudget;
use ares_context::pack::markdown::ToMarkdown;
use std::sync::Arc;

use ares_store::db::Store;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing ARES Context Engine Self-Test...");

    let db_path = "ares_memory.db";
    let store = Store::open(Path::new(db_path))?;
    let graph_repo = Arc::new(SqliteGraphRepository::new(store.clone()));
    let memory_repo = Arc::new(SqliteMemoryRepository::new(store.clone()));

    let project_id = ProjectId::from("Eswar-426/ARES-MEMORY-OS");

    let config = TraversalConfig {
        max_depth: 3,
        max_neighbors: 10,
        max_results: 20,
    };

    let engine = ContextEngine::new(
        project_id,
        graph_repo,
        memory_repo,
        config,
    );

    let budget = ContextBudget::default();
    let builder = ContextPackBuilder::new(budget);

    let test_queries = vec![
        "explain crates/ares-scanner/src/scanner.rs",
        "what is the impact of changing crates/ares-store/src/repositories/graph.rs",
        "trace dependencies for parser",
        "summarize repository architecture",
    ];

    for query in test_queries {
        println!("\n========================================");
        println!("QUERY: {}", query);
        println!("========================================");

        match engine.resolve_query(query).await {
            Ok(bundle) => {
                let pack = builder.build(bundle);
                if let Err(e) = ContextPackValidator::validate(&pack) {
                    println!("Validation Warning: {}", e);
                }

                println!("Metrics: {:?}", pack.metrics);
                println!("Ranked Nodes Returned: {}", pack.relevant_nodes.len());
                println!("Bundle Summary: {}", pack.summary);
                println!("Impact Reports: {}", pack.impact_analysis.len());
                println!("Explanations: {}", pack.relevant_files.len());
                println!("Dependencies Traced: {}", pack.dependency_trace.len());
                println!("Insights: {}", pack.memory_snippets.len());
            }
            Err(e) => {
                println!("Error resolving query: {:?}", e);
            }
        }
    }

    Ok(())
}
