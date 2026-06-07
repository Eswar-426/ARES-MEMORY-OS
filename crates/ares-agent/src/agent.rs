use crate::config::AgentConfig;
use ares_core::AresError;
use tracing::info;

/// The ARES Local Agent — manages lifecycle of IPC server + scanner.
/// Implemented progressively across Weeks 4–7.
pub struct Agent {
    _config: AgentConfig,
}

impl Agent {
    pub async fn new(config: AgentConfig) -> Result<Self, AresError> {
        info!(project = %config.project_path, "Agent initialized");
        Ok(Self { _config: config })
    }

    /// Main event loop — starts IPC server and waits for shutdown signal.
    pub async fn run(&mut self) -> Result<(), AresError> {
        // TODO Week 4: Start IPC server
        // TODO Week 4: Register project
        // TODO Week 6: Start scanner
        // TODO Week 7: Start file watcher

        info!("Agent running — press Ctrl+C to stop");

        // Graceful shutdown on Ctrl+C
        tokio::signal::ctrl_c().await.map_err(AresError::Io)?;
        info!("Agent shutting down");
        Ok(())
    }
}
