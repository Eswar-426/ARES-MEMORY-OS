use anyhow::Result;
use ares_benchmark::agent::MockAgentProvider;
use ares_benchmark::report::BenchmarkReport;
use ares_benchmark::runner::BenchmarkRunner;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The task to benchmark (e.g., "Add OAuth")
    #[arg(short, long)]
    task: String,

    /// The repository to run the benchmark against
    #[arg(short, long)]
    repo: PathBuf,

    /// The agent provider to use (mock, gemini, claude, openai)
    #[arg(short, long, default_value = "mock")]
    provider: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    println!("🚀 Starting ARES Benchmark Engine...");
    println!("Task: {}", args.task);
    println!("Repo: {}", args.repo.display());
    println!("Provider: {}\n", args.provider);

    // Initialize provider (currently only mock is wired up for the MVP)
    let provider = Box::new(MockAgentProvider::new());

    // Initialize ARES App State (if available, otherwise it'll just simulate)
    // For full testing, we would instantiate a real AppState.
    let app_state = None;

    let runner = BenchmarkRunner::new(args.repo.clone(), provider, app_state);

    let results = runner.run_task(&args.task).await?;

    let report = BenchmarkReport::new(
        args.task.clone(),
        args.repo.to_string_lossy().to_string(),
        results,
    );

    println!("\n====================================");
    println!("{}", report.to_markdown());
    println!("====================================\n");

    // Also write to file
    let out_dir = std::env::current_dir()?.join("reports");
    std::fs::create_dir_all(&out_dir)?;

    let md_path = out_dir.join("benchmark_report.md");
    std::fs::write(&md_path, report.to_markdown())?;
    println!("Saved Markdown report to: {}", md_path.display());

    let json_path = out_dir.join("benchmark_report.json");
    std::fs::write(&json_path, report.to_json())?;
    println!("Saved JSON report to: {}", json_path.display());

    Ok(())
}
