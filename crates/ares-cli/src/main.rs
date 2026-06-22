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

    /// Generates a complete Governance Report (Coverage, Debt, Health, Maturity, Drift, Confidence)
    Report {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
    },
    
    /// Generates Coverage Metrics
    Coverage { #[arg(long)] json: bool, #[arg(long)] markdown: bool },
    
    /// Generates Memory Debt Metrics
    Debt { #[arg(long)] json: bool, #[arg(long)] markdown: bool },
    
    /// Generates Memory Health Metrics
    Health { #[arg(long)] json: bool, #[arg(long)] markdown: bool },
    
    /// Generates Memory Maturity Level
    Maturity { #[arg(long)] json: bool, #[arg(long)] markdown: bool },
    
    /// Generates Memory Drift Metrics
    Drift { #[arg(long)] json: bool, #[arg(long)] markdown: bool },
    
    /// Generates Traceability Confidence
    Confidence { #[arg(long)] json: bool, #[arg(long)] markdown: bool },
    
    /// Validates PR memory impact as a CI gatekeeper
    Check {
        #[arg(long, help = "Baseline branch or snapshot")]
        baseline: Option<String>,
    },
    
    /// Snapshot subcommands
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCommands,
    },
    
    /// Runs the MemoryOS benchmark suite against known frameworks
    Benchmark {
        #[arg(long)]
        synthetic: bool,
        #[arg(long)]
        real: bool,
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum SnapshotCommands {
    /// Creates a new memory snapshot and saves it
    Create {
        #[arg(long, default_value = "snapshot.json")]
        out: String,
    },
    /// Compares current memory to a previous snapshot
    Compare {
        #[arg(long)]
        baseline: String,
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
                GovernanceCommands::Report { json, markdown } => {
                    commands::governance::execute_report(*json, *markdown).await?;
                }
                GovernanceCommands::Coverage { json, markdown } => {
                    commands::governance::execute_coverage(*json, *markdown).await?;
                }
                GovernanceCommands::Debt { json, markdown } => {
                    commands::governance::execute_debt(*json, *markdown).await?;
                }
                GovernanceCommands::Health { json, markdown } => {
                    commands::governance::execute_health(*json, *markdown).await?;
                }
                GovernanceCommands::Maturity { json, markdown } => {
                    commands::governance::execute_maturity(*json, *markdown).await?;
                }
                GovernanceCommands::Drift { json, markdown } => {
                    commands::governance::execute_drift(*json, *markdown).await?;
                }
                GovernanceCommands::Confidence { json, markdown } => {
                    commands::governance::execute_confidence(*json, *markdown).await?;
                }
                GovernanceCommands::Check { baseline } => {
                    commands::governance::execute_check(baseline.clone()).await?;
                }
                GovernanceCommands::Snapshot { action } => {
                    match action {
                        SnapshotCommands::Create { out } => {
                            commands::governance::execute_snapshot_create(out.clone()).await?;
                        }
                        SnapshotCommands::Compare { baseline } => {
                            commands::governance::execute_snapshot_compare(baseline.clone()).await?;
                        }
                    }
                }
                GovernanceCommands::Benchmark { synthetic, real, all } => {
                    commands::governance::execute_benchmark(*synthetic, *real, *all).await?;
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
