#![allow(unused_assignments)]
pub mod commands;
use ares_core::AresError;
use ares_memory_server::builder::RepositoryBuilder;
use ares_memory_server::initializer::RepositoryInitializer;
use ares_memory_server::scanner::RepositoryScanner;
use clap::{Parser, Subcommand};
use std::env;
// We would initialize full dependencies here

#[derive(Parser)]
#[command(name = "ares")]
#[command(
    about = "ARES Repository Memory Operating System CLI",
    long_about = "ARES Repository Memory Operating System CLI\n\nQuick Start:\n  1. Initialize repository:\n     $ ares ingest .\n\n  2. Check system health:\n     $ ares doctor\n\n  3. Evaluate a pull request:\n     $ ares governance pr-check\n\n  4. Start MCP server (for IDEs):\n     $ ares-mcp\n"
)]
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
    /// Candidates & Intelligence Engine
    Candidates {
        #[command(subcommand)]
        action: CandidatesCommands,
    },
    /// Simulation OS commands
    Simulate {
        #[command(subcommand)]
        action: SimulateCommands,
    },
    /// Repository Ingestion
    Ingest(commands::ingest::IngestArgs),
    /// Traceability Analysis
    Traceability {
        #[command(subcommand)]
        action: TraceabilityCommands,
    },

    /// System Health Check
    Doctor,

    /// Compact the database
    Compact,

    /// Run real-world engine benchmarks
    Benchmark,

    /// Run the interactive ARES Demo
    Demo,

    // --- P3.4 Reasoning Engines ---
    /// Explain decisions and rationale
    Explain {
        #[command(subcommand)]
        action: ExplainCommands,
    },
    /// Why Engine: Explain lineage of a node
    Why {
        target_type: String, // e.g. "file", "architecture"
        target_id: String,
    },
    /// Impact Engine: Analyze downstream impact
    Impact {
        #[command(subcommand)]
        action: ImpactCommands,
    },
    /// Breakage Engine: What breaks due to a change?
    WhatBreaks { target_id: String },
    /// Memory Gap Detection
    Gaps,

    // --- P11 Production Memory Server ---
    /// Initialize ARES for this repository
    Init,
    /// Scan the repository
    Scan,
    /// Build the memory graph
    Build,
    /// Bootstrap missing memory candidates
    Bootstrap,
    /// Serve the API
    Serve,
}

#[derive(Subcommand)]
enum GovernanceCommands {
    /// Lists active policy exemptions
    Exemptions,

    /// Evaluates a Pull Request impact against a base graph state
    PrCheck {
        #[arg(
            long,
            help = "Path to the base MemorySnapshot JSON file. If omitted, uses historical DB snapshot."
        )]
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
    Coverage {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
    },

    /// Generates Memory Debt Metrics
    Debt {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
    },

    /// Generates Memory Health Metrics
    Health {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
    },

    /// Generates Memory Maturity Level
    Maturity {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
    },

    /// Generates Memory Drift Metrics
    Drift {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
    },

    /// Generates Traceability Confidence
    Confidence {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
    },

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
    Requirement { id: String, action: String },
    /// Simulates changes to a code component
    Code { path: String, action: String },
}

#[derive(Subcommand)]
pub enum TraceabilityCommands {
    /// Explain the traceability graph for a given path
    Explain {
        #[arg(help = "The path or ID to explain traceability for")]
        path: String,
    },
}

#[derive(Subcommand)]
pub enum ExplainCommands {
    /// Explain a specific decision
    Decision { id: String },
}

#[derive(Subcommand)]
pub enum ImpactCommands {
    /// Analyze downstream impact
    Analyze { target_id: String },
}

#[derive(Subcommand)]
enum CandidatesCommands {
    /// List pending candidates
    List,
    /// Show details of a specific candidate
    Show { id: String },
    /// Accept and promote a candidate
    Accept { id: String },
    /// Reject a candidate
    Reject { id: String },
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
        #[arg(
            long,
            help = "Output file path",
            default_value = "memory_snapshot.json"
        )]
        out: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), AresError> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Memory { action } => match action {
            MemoryCommands::Validate {
                strict,
                json,
                sarif,
                ci,
            } => {
                commands::memory::execute_validate(*strict, *json, *sarif, *ci).await?;
            }
            MemoryCommands::Export { out } => {
                commands::memory::execute_export(out).await?;
            }
        },
        Commands::Governance { action } => match action {
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
            GovernanceCommands::Snapshot { action } => match action {
                SnapshotCommands::Create { out } => {
                    commands::governance::execute_snapshot_create(out.clone()).await?;
                }
                SnapshotCommands::Compare { baseline } => {
                    commands::governance::execute_snapshot_compare(baseline.clone()).await?;
                }
            },
            GovernanceCommands::Benchmark {
                synthetic,
                real,
                all,
            } => {
                commands::governance::execute_benchmark(*synthetic, *real, *all).await?;
            }
        },
        Commands::Simulate { action } => match action {
            SimulateCommands::Requirement { id, action } => {
                commands::simulate::execute_simulate_requirement(id.clone(), action.clone())
                    .await?;
            }
            SimulateCommands::Code { path, action } => {
                commands::simulate::execute_simulate_code(path.clone(), action.clone()).await?;
            }
        },
        Commands::Candidates { action } => match action {
            CandidatesCommands::List => {
                commands::candidates::execute_list().await?;
            }
            CandidatesCommands::Show { id } => {
                commands::candidates::execute_show(id.clone()).await?;
            }
            CandidatesCommands::Accept { id } => {
                commands::candidates::execute_accept(id.clone()).await?;
            }
            CandidatesCommands::Reject { id } => {
                commands::candidates::execute_reject(id.clone()).await?;
            }
        },
        Commands::Traceability { action } => {
            commands::traceability::handle_traceability(action).await?;
        }
        Commands::Ingest(args) => {
            commands::ingest::handle_ingest(args.clone()).await?;
        }
        Commands::Doctor => {
            commands::doctor::execute_doctor().await?;
        }
        Commands::Compact => {
            commands::compact::execute_compact().await?;
        }
        Commands::Benchmark => {
            commands::benchmark::run_real_benchmark().await?;
        }
        Commands::Demo => {
            println!("ARES Demo\n");
            println!("Step 1\nRepository loaded\n?\n");
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            println!("Step 2\nQuestion submitted: \"Why does PaymentProvider exist?\"\n?\n");
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            println!("Step 3\nPlanner executing...\n?\n");
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            println!("Step 4\nEvidence gathered from 6 engines\n?\n");
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            println!("Step 5\nAnswer generated\n?\n");
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            println!("Step 6\nOpen Graph Explorer\n?\n");
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            println!("Step 7\nOpen Node Inspector for PaymentProvider\n?\n");
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            println!("Step 8\nRun Impact Analysis\n?\n");
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            println!("Complete\n\nARES is ready.");
        }
        Commands::Explain { action } => match action {
            ExplainCommands::Decision { id } => {
                commands::reasoning::handle_explain_decision(id).await?;
            }
        },
        Commands::Why {
            target_type,
            target_id,
        } => {
            commands::reasoning::handle_why(target_type, target_id).await?;
        }
        Commands::Impact { action } => match action {
            ImpactCommands::Analyze { target_id } => {
                commands::reasoning::handle_impact(target_id).await?;
            }
        },
        Commands::WhatBreaks { target_id } => {
            commands::reasoning::handle_what_breaks(target_id).await?;
        }
        Commands::Gaps => {
            commands::reasoning::handle_gaps().await?;
        }
        Commands::Init => {
            let path = env::current_dir().unwrap();
            RepositoryInitializer::init(&path)?;
            // Initialize the store to create ares.db and run schema migrations
            let db_path = path.join(".ares").join("ares.db");
            let store = ares_store::Store::open(&db_path)?;
            let project_id_str = path.file_name().and_then(|n| n.to_str()).unwrap_or("project");
            store.run_migrations(project_id_str)?;
            println!(
                "Initialized ARES Repository Memory Operating System in {:?}",
                path
            );
        }
        Commands::Scan => {
            let path = env::current_dir().unwrap();
            let db_path = path.join(".ares").join("ares.db");
            let store = ares_store::Store::open(&db_path)?;
            let project_id_str = path.file_name().and_then(|n| n.to_str()).unwrap_or("project");
            let project_id = ares_core::ProjectId::from(project_id_str);
            
            // Ensure project exists in DB
            let project_repo = ares_store::repositories::project::SqliteProjectRepository::new(store.clone());
            if project_repo.get_by_id(&project_id)?.is_none() {
                let now = ares_core::types::event::now_micros();
                let project = ares_core::Project {
                    id: project_id.clone(),
                    name: project_id_str.to_string(),
                    description: "Auto-initialized project".to_string(),
                    root_path: path.to_string_lossy().to_string(),
                    primary_language: "".to_string(),
                    domain: "".to_string(),
                    maturity: ares_core::ProjectMaturity::Greenfield,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                };
                project_repo.create(&project)?;
            }
            
            let graph_repo = std::sync::Arc::new(ares_store::repositories::graph::SqliteGraphRepository::new(store));
            let scanner = ares_scanner::Scanner::new(graph_repo);
            
            let _report = scanner.full_scan(&project_id, &path)?;
            println!("Repository scanning completed.");
        }
        Commands::Build => {
            let path = env::current_dir().unwrap();
            RepositoryBuilder::build(&path)?;
            println!("Memory building completed.");
        }
        Commands::Bootstrap => {
            let path = env::current_dir().unwrap();
            println!("Bootstrapping memory candidates...");
            // Execute bootstrap logic
            commands::bootstrap::execute_bootstrap(&path).await?;
            println!("Memory bootstrap completed.");
        }
        Commands::Serve => {
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
            println!("API Server starting on http://{}", addr);
            ares_memory_server::server::serve(addr)
                .await
                .map_err(|e| AresError::validation(e.to_string()))?;
        }
    }

    Ok(())
}

pub fn get_default_project_id() -> ares_core::ProjectId {
    // Derive from current working directory
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if let Some(name) = cwd.file_name().and_then(|n| n.to_str()) {
        if !name.is_empty() && name != "." && name != ".." {
            return ares_core::ProjectId::from(name);
        }
    }

    // Fallback to project-<uuid> if no valid workspace name
    let fallback = format!("project-{}", uuid::Uuid::now_v7());
    ares_core::ProjectId::from(fallback.as_str())
}
