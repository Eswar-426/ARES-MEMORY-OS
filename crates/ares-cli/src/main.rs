pub mod commands;

use clap::{Parser, Subcommand};
use ares_core::AresError;
// We would initialize full dependencies here

#[derive(Parser)]
#[command(name = "ares")]
#[command(about = "ARES Repository Memory Operating System CLI", long_about = None)]
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
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Validates the Memory OS certification status
    Validate,
}

#[tokio::main]
async fn main() -> Result<(), AresError> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Memory { action } => {
            match action {
                MemoryCommands::Validate => {
                    commands::memory::execute_validate().await?;
                }
            }
        }
    }

    Ok(())
}
