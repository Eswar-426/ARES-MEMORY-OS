use super::AgentRunner;
use crate::models::{AgentRunResult, AgentType, ArenaTask};
use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;

pub struct PlannerAgentStub {}

#[async_trait]
impl AgentRunner for PlannerAgentStub {
    async fn run(&self, task: &ArenaTask) -> Result<AgentRunResult> {
        let start = Instant::now();

        // Stub simulates task decomposition and perfectly finding the expected files
        let mut retrieved_files = task.expected_files.clone();
        let retrieved_components = task.expected_components.clone();
        
        // Add a little bit of noise just to make it realistic (a planner isn't 100% perfect always, but for now we mock it as very high performing)
        if !retrieved_files.contains(&"src/lib.rs".to_string()) {
            retrieved_files.push("src/lib.rs".to_string());
        }
        let latency_ms = start.elapsed().as_millis() as u64 + 1500; // Simulated latency of planner thinking

        Ok(AgentRunResult {
            task_id: task.id.clone(),
            agent_type: AgentType::Planner,
            response: format!("Planned changes for {} tasks.", retrieved_files.len()),
            latency_ms,
            context_nodes_used: retrieved_components.len(),
            retrieved_files,
            retrieved_components,
            precision_score: 0.0,
            recall_score: 0.0,
            confidence_score: 0.0,
            overall_score: 0.0,
            graph_coverage: 1.0, 
            context_efficiency: 1.0, 
            reasoning_accuracy: 1.0,
            reasoning_coverage: 1.0,
            reasoning_precision: 1.0,
        })
    }
}
