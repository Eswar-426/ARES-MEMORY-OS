//! ARES Local Agent — main entry point.
//!
//! The agent is a long-running daemon that:
//! 1. Opens the project SQLite database
//! 2. Starts the IPC server (Unix socket / Named Pipe)
//! 3. Manages the project scanner
//! 4. Emits events to in-process subscribers

mod agent;
mod config;
mod ipc;
mod services;

use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "ares-agent",
    about = "ARES MemoryOS Local Agent",
    version
)]
struct Args {
    /// Path to the project root (defaults to current directory)
    #[arg(short, long)]
    project_path: Option<String>,

    /// Run in detached background mode
    #[arg(long)]
    detach: bool,

    /// Just check if the agent can start and exit
    #[arg(long)]
    check: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", env = "ARES_LOG")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize structured logging
    let filter = EnvFilter::try_new(&args.log_level)
        .unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    info!(version = env!("CARGO_PKG_VERSION"), "ARES Agent starting");

    if args.check {
        println!("ARES Agent OK — version {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let project_path = args.project_path
        .unwrap_or_else(|| std::env::current_dir()
            .expect("Cannot determine current directory")
            .to_string_lossy()
            .to_string());

    // Load config and start agent
    let config = config::AgentConfig::load(&project_path)?;
    let mut agent = agent::Agent::new(config).await?;
    agent.run().await?;

    Ok(())
}
