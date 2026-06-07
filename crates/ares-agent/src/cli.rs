// ARES CLI — `ares` binary entry point (Week 16)
// Provides developer-facing terminal commands.
// Communicates with the running ares-agent via IPC.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ares", about = "ARES MemoryOS CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize ARES for a project
    Init {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Scan the project codebase
    Scan {
        #[arg(long)]
        full: bool,
    },
    /// Memory operations
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },
    /// Decision operations
    Decision {
        #[command(subcommand)]
        action: DecisionAction,
    },
    /// Retrieve context for a query
    Context {
        query: String,
        #[arg(long, default_value = "4000")]
        token_budget: u32,
    },
    /// Check ARES environment health
    Doctor,
}

#[derive(Subcommand, Debug)]
enum MemoryAction {
    List {
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        since: Option<String>,
    },
    Search {
        query: String,
    },
    Get {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum DecisionAction {
    Create,
    List,
    Get { id: String },
}

fn main() {
    let _cli = Cli::parse();
    // TODO Week 16: Implement all commands
    println!("ARES CLI v{} — commands coming in Week 16", env!("CARGO_PKG_VERSION"));
}
