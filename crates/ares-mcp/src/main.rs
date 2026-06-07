use ares_agent::config::AgentConfig;
use ares_app::AppState;
use ares_mcp::server::run_stdio_server;
use ares_mcp::McpServer;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Observability Setup
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,ares_mcp=debug".into()),
        )
        // For MCP over stdio, we MUST write logs to stderr instead of stdout
        // because stdout is used for JSON-RPC messages!
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting ARES MCP Server"
    );

    let project_path = std::env::current_dir()
        .expect("Cannot determine current directory")
        .to_string_lossy()
        .to_string();

    let config = AgentConfig::load(&project_path)?;
    let app_state = AppState::new(config).await?;

    let mcp_server = Arc::new(McpServer::new(app_state));

    run_stdio_server(mcp_server).await?;

    Ok(())
}
