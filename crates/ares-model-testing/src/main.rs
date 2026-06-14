use anyhow::Result;
use ares_model_testing::{
    pipeline::{BenchmarkMode, ContinuityEngine, PipelineType},
    registry::{ModelCatalog, ProviderRegistry},
    report::ContinuityReport,
    scenarios::get_test_scenarios,
};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    println!("Starting ARES Continuity Validation Suite...\n");

    let openrouter_key = env::var("OPENROUTER_API_KEY").ok();
    let nvidia_key = env::var("NVIDIA_API_KEY").ok();
    let gemini_key = env::var("GEMINI_API_KEY").ok();
    let groq_key = env::var("GROQ_API_KEY").ok();

    if openrouter_key.is_none() && nvidia_key.is_none() {
        println!(
            "❌ Neither OPENROUTER_API_KEY nor NVIDIA_API_KEY found. Please check your .env file."
        );
        return Ok(());
    }

    let engine = ContinuityEngine::new();

    // Load Model Catalog
    let catalog_path = Path::new("models.yaml");
    let catalog = ModelCatalog::load(catalog_path).expect("Failed to load models.yaml");

    let mode_str = env::var("BENCHMARK_MODE").unwrap_or_else(|_| "simulated".to_string());
    let benchmark_mode = if mode_str == "real" {
        println!("🚀 Running in REAL API Mode (Disk I/O + Real Models + ARES Scanner)");
        BenchmarkMode::RealApi
    } else {
        println!("🚀 Running in SIMULATED Mode (In-Memory)");
        BenchmarkMode::Simulated
    };

    let mut report = ContinuityReport::new();
    let scenarios = get_test_scenarios();

    // Setup Provider Registry using dynamic discovery
    let mut registry = ProviderRegistry::new(
        catalog,
        openrouter_key,
        gemini_key,
        nvidia_key,
        groq_key,
        "free".to_string(),
    );

    // Phase 2: Health Checks
    registry.health_check().await;

    // Phase 3: Provider Discovery & Dynamic Chain Assembly
    registry.build_dynamic_chains().await;

    for scenario in &scenarios {
        // Run Baseline (No ARES)
        engine
            .run_chain(
                scenario,
                &mut report,
                benchmark_mode,
                PipelineType::Baseline,
                &mut registry,
            )
            .await;
        // Run ARES (With Snapshot & Context Generation)
        engine
            .run_chain(
                scenario,
                &mut report,
                benchmark_mode,
                PipelineType::Ares,
                &mut registry,
            )
            .await;
    }

    report.print_report();

    // ---------------------------------------------------------
    // Phase 4: Save Telemetry to SQLite
    // ---------------------------------------------------------
    let db_path = std::env::var("ARES_TELEMETRY_DB_PATH")
        .unwrap_or_else(|_| "scratch/benchmark.db".to_string());

    println!("💾 Saving Telemetry to {}...", db_path);
    let store = ares_store::db::Store::open(std::path::Path::new(&db_path))
        .expect("Failed to open Telemetry Database");
    let repo = ares_store::repositories::telemetry::TelemetryRepository::new(&store);

    let continuity_score = report.overall_score();
    let provider_health =
        serde_json::to_value(registry.get_health_map()).unwrap_or(serde_json::Value::Null);
    let dynamic_chains =
        serde_json::to_value(registry.get_dynamic_chains()).unwrap_or(serde_json::Value::Null);
    let fallback_events = serde_json::json!([]); // Empty for MVP

    match repo.save_report(
        "model-testing",
        continuity_score,
        provider_health,
        fallback_events,
        dynamic_chains,
    ) {
        Ok(id) => println!("✅ Telemetry Report saved: {}", id),
        Err(e) => println!("❌ Failed to save Telemetry Report: {}", e),
    }

    Ok(())
}
