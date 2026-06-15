use ares_benchmark::context::dataset::{BenchmarkDataset, BenchmarkQuery};
use ares_benchmark::context::evaluator::{ContextEvaluator, ContextBenchmarkReport};
use ares_context::context_engine::ContextEngine;
use ares_context::pack::{ContextPackBuilder, ContextPackValidator};
use ares_context::models::pack::ContextBudget;
use ares_context::traversal::TraversalConfig;
use ares_core::ProjectId;
use ares_store::db::Store;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading Golden Dataset from crates/ares-benchmark/context_dataset.json...");
    let dataset = BenchmarkDataset::load_from_file("crates/ares-benchmark/context_dataset.json")?;
    
    println!("Initializing ARES Context Engine...");
    let store = Store::open(Path::new("ares_memory.db"))?;
    let graph_repo = Arc::new(SqliteGraphRepository::new(store.clone()));
    let memory_repo = Arc::new(SqliteMemoryRepository::new(store.clone()));

    let project_id = ProjectId::from("Eswar-426/ARES-MEMORY-OS");
    let config = TraversalConfig {
        max_depth: 3,
        max_neighbors: 10,
        max_results: 50,
    };

    let engine = ContextEngine::new(
        project_id,
        graph_repo,
        memory_repo,
        config,
    );

    let budget = ContextBudget::default();
    let builder = ContextPackBuilder::new(budget);

    let mut evaluations = Vec::new();

    for query in &dataset.queries {
        println!("Evaluating: {}", query.query);
        match engine.resolve_query(&query.query).await {
            Ok(bundle) => {
                let pack = builder.build(bundle);
                
                if let Err(e) = ContextPackValidator::validate(&pack) {
                    println!("  [!] Validation Warning: {}", e);
                }

                let eval = ContextEvaluator::evaluate_pack(query, &pack);
                println!("  -> Passed: {}, Latency: {}ms", eval.passed, eval.latency_ms);
                evaluations.push(eval);
            }
            Err(e) => {
                println!("  [X] Failed to resolve query: {:?}", e);
            }
        }
    }

    let report = ContextEvaluator::generate_report(evaluations);
    
    println!("\n=========================================");
    println!("Benchmark Report");
    println!("=========================================");
    println!("Total Queries: {}", report.total_queries);
    println!("Passing: {}/{}", report.passing_queries, report.total_queries);
    println!("Avg Recall: {:.2}", report.avg_recall);
    println!("Avg Precision: {:.2}", report.avg_precision);
    println!("Avg Latency: {}ms", report.avg_latency_ms);
    println!("=========================================\n");

    std::fs::create_dir_all("artifacts/benchmark")?;
    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write("artifacts/benchmark/context_report_v0.9.json", report_json)?;
    println!("Report saved to artifacts/benchmark/context_report_v0.9.json");

    Ok(())
}
