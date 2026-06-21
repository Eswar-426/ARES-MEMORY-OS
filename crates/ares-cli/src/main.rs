pub mod commands;

use clap::{Parser, Subcommand};
use ares_core::AresError;
// We would initialize full dependencies here

#[derive(Parser)]
#[command(name = "ares")]
#[command(about = "ARES Repository Memory Operating System CLI", long_about = "ARES Repository Memory Operating System CLI\n\nQuick Start:\n  1. Initialize repository:\n     $ ares ingest .\n\n  2. Check system health:\n     $ ares doctor\n\n  3. Evaluate a pull request:\n     $ ares governance pr-check\n\n  4. Start MCP server (for IDEs):\n     $ ares-mcp\n")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Memory OS commands
    Memory {
        #[command(subcommand)]
        action: MemoryCommands,
    },
    /// Governance OS commands
    Governance {
        #[command(subcommand)]
        action: GovernanceCommands,
    },
    /// Simulation OS commands
    Simulate {
        #[command(subcommand)]
        action: SimulateCommands,
    },
    /// Repository Ingestion
    Ingest(commands::ingest::IngestArgs),
    
    /// System Health Check
    Doctor,
}

#[derive(Subcommand)]
enum GovernanceCommands {
    /// Lists active policy exemptions
    Exemptions,
    
    /// Evaluates a Pull Request impact against a base graph state
    PrCheck {
        #[arg(long, help = "Path to the base MemorySnapshot JSON file. If omitted, uses historical DB snapshot.")]
        base_report: Option<String>,
    },
}

#[derive(Subcommand)]
enum SimulateCommands {
    /// Simulates changes to a requirement
    Requirement {
        id: String,
        action: String,
    },
    /// Simulates changes to a code component
    Code {
        path: String,
        action: String,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Validates the Memory OS certification status
    Validate {
        #[arg(long, help = "Exit with code 1 if blocking violations are present")]
        strict: bool,
        #[arg(long, help = "Output the results as JSON")]
        json: bool,
        #[arg(long, help = "Export results as SARIF to governance.sarif")]
        sarif: bool,
        #[arg(long, help = "CI mode: outputs JSON and strictly enforces exit codes")]
        ci: bool,
    },
    /// Exports the current Memory Graph and Certification state to a JSON snapshot
    Export {
        #[arg(long, help = "Output file path", default_value = "memory_snapshot.json")]
        out: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), AresError> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Memory { action } => {
            match action {
                MemoryCommands::Validate { strict, json, sarif, ci } => {
                    commands::memory::execute_validate(*strict, *json, *sarif, *ci).await?;
                }
                MemoryCommands::Export { out } => {
                    commands::memory::execute_export(out).await?;
                }
            }
        }
        Commands::Governance { action } => {
            match action {
                GovernanceCommands::Exemptions => {
                    commands::governance::execute_exemptions().await?;
                }
                GovernanceCommands::PrCheck { base_report } => {
                    commands::governance::execute_pr_check(base_report.clone()).await?;
                }
            }
        }
        Commands::Simulate { action } => {
            match action {
                SimulateCommands::Requirement { id, action } => {
                    commands::simulate::execute_simulate_requirement(id.clone(), action.clone()).await?;
                }
                SimulateCommands::Code { path, action } => {
                    commands::simulate::execute_simulate_code(path.clone(), action.clone()).await?;
                }
            }
        }
        Commands::Ingest(args) => {
            commands::ingest::handle_ingest(args.clone()).await?;
        }
        Commands::Doctor => {
            commands::doctor::execute_doctor().await?;
        }
    }

    Ok(())
}
