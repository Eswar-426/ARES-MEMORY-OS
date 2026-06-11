use crate::models::{AgentId, MissionId, TaskId};
use crate::workflow::{MissionDag, MissionNode};

pub mod autonomous;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplanningTrigger {
    TaskFailure(TaskId),
    AgentFailure(AgentId),
    ToolFailure(String),
    BudgetExhaustion,
    LowConfidence,
    MissingInformation,
    Timeout,
    QualityBelowThreshold(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplanningAction {
    ModifyPlan(MissionDag),         // A new partial or full DAG
    ReplaceAgent(AgentId, AgentId), // Old, New
    SwitchModel(AgentId, String),   // Agent, New Model
    RetryStrategy(TaskId),
    Escalate(String),
    SplitTask(TaskId, Vec<MissionNode>),
    RebuildDag(MissionDag),
    ChangeModel(AgentId, String),
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
        match trigger {
            ReplanningTrigger::TaskFailure(task_id) => Ok(ReplanningAction::RetryStrategy(task_id)),
            ReplanningTrigger::AgentFailure(agent_id) => Ok(ReplanningAction::Escalate(format!(
                "Agent {} failed irreparably",
                agent_id.0
            ))),
            ReplanningTrigger::ToolFailure(tool_name) => Ok(ReplanningAction::Escalate(format!(
                "Critical tool {} failed",
                tool_name
            ))),
            ReplanningTrigger::BudgetExhaustion => {
                Ok(ReplanningAction::Escalate("Budget exhausted".into()))
            }
            ReplanningTrigger::Timeout => {
                Ok(ReplanningAction::Escalate("Mission timed out".into()))
            }
            ReplanningTrigger::QualityBelowThreshold(_threshold_pct) => {
                // Rebuild the DAG with potential modifications
                Ok(ReplanningAction::RebuildDag(current_dag.clone()))
            }
            ReplanningTrigger::LowConfidence | ReplanningTrigger::MissingInformation => {
                Ok(ReplanningAction::ModifyPlan(current_dag.clone()))
            }
        }
    }
}
