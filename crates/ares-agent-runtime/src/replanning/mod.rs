use crate::models::{AgentId, MissionId, TaskId};
use crate::workflow::MissionDag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplanningTrigger {
    TaskFailure(TaskId),
    AgentFailure(AgentId),
    ToolFailure(String),
    BudgetExhaustion,
    LowConfidence,
    MissingInformation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplanningAction {
    ModifyPlan(MissionDag),         // A new partial or full DAG
    ReplaceAgent(AgentId, AgentId), // Old, New
    SwitchModel(AgentId, String),   // Agent, New Model
    RetryStrategy(TaskId),
    Escalate(String),
}

pub struct Replanner {
    // Requires intelligence client to generate new plans
}

impl Default for Replanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Replanner {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle_trigger(
        &self,
        _mission_id: MissionId,
        current_dag: &MissionDag,
        trigger: ReplanningTrigger,
    ) -> Result<ReplanningAction, String> {
        // In a real implementation, this would prompt the Intelligence layer
        // to figure out how to modify the plan based on the failure context.
        match trigger {
            ReplanningTrigger::TaskFailure(task_id) => {
                // Heuristic or AI-driven decision: retry the task
                Ok(ReplanningAction::RetryStrategy(task_id))
            }
            ReplanningTrigger::AgentFailure(agent_id) => {
                // Heuristic: Escalate or try replacing
                Ok(ReplanningAction::Escalate(format!(
                    "Agent {} failed irreparably",
                    agent_id.0
                )))
            }
            ReplanningTrigger::ToolFailure(tool_name) => Ok(ReplanningAction::Escalate(format!(
                "Critical tool {} failed",
                tool_name
            ))),
            ReplanningTrigger::BudgetExhaustion => {
                Ok(ReplanningAction::Escalate("Budget exhausted".into()))
            }
            ReplanningTrigger::LowConfidence | ReplanningTrigger::MissingInformation => {
                // Example of modify plan: return current DAG for now
                Ok(ReplanningAction::ModifyPlan(current_dag.clone()))
            }
        }
    }
}
