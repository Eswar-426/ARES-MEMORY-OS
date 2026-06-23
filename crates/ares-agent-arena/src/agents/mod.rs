pub mod baseline;
pub mod context_aware;
pub mod enhanced;
pub mod planner;

use crate::models::{AgentRunResult, ArenaTask};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait AgentRunner {
    async fn run(&self, task: &ArenaTask) -> Result<AgentRunResult>;
}
