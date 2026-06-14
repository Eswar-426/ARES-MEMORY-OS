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
    /// Context generator operations
    Context {
        #[command(subcommand)]
        action: ContextAction,
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
enum ContextAction {
    Query {
        query: String,
        #[arg(long, default_value = "4000")]
        token_budget: u32,
    },
    Export {
        #[arg(long)]
        project: String,
        #[arg(long)]
        clipboard: bool,
    },
}

#[derive(Subcommand, Debug)]
enum DecisionAction {
    Create,
    List,
    Get { id: String },
}

fn main() {
    let cli = Cli::parse();

    // Use tokio for async commands
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        match cli.command {
            Commands::Context { action } => match action {
                ContextAction::Export { project, clipboard } => {
                    println!("ARES: Exporting Portable Context for project {}...", project);

                    let client = reqwest::Client::new();
                    let url = format!("http://127.0.0.1:3000/api/v1/project/{}/context", project);

                    match client.get(&url).send().await {
                        Ok(res) if res.status().is_success() => {
                            if let Ok(json) = res.json::<serde_json::Value>().await {
                                let markdown = json["text"].as_str().unwrap_or_default();
                                let tokens = json["estimated_tokens"].as_u64().unwrap_or(0);


                                if clipboard {
                                    if let Ok(mut board) = arboard::Clipboard::new() {
                                        if let Err(e) = board.set_text(markdown) {
                                            println!("❌ Failed to copy to clipboard: {}", e);
                                        } else {
                                            println!("✅ Successfully copied context to clipboard! (~{} tokens)", tokens);
                                        }
                                    } else {
                                        println!("❌ Failed to access clipboard.");
                                    }
                                } else {
                                    println!("\n{}\n", markdown);
                                    println!("✅ Context generated (~{} tokens). Use --clipboard to copy it.", tokens);
                                }
                            }
                        }
                        Ok(res) => {
                            println!("❌ API Error: {}", res.status());
                        }
                        Err(e) => {
                            println!("❌ Failed to connect to ARES Memory OS daemon. Is it running? Error: {}", e);
                        }
                    }
                }
                _ => {
                    println!("Command not fully implemented yet.");
                }
            },
            _ => {
                println!(
                    "ARES CLI v{} — command not implemented yet",
                    env!("CARGO_PKG_VERSION")
                );
            }
        }
    });
}
