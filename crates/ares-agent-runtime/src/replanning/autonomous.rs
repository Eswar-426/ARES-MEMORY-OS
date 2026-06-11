use crate::evaluation::models::MissionScore;
use crate::models::AgentRole;
use crate::models::{AgentId, MissionId, TaskId};
use crate::reflection::mission_reflection::MissionReflection;
use crate::replanning::{ReplanningAction, ReplanningTrigger};
use crate::workflow::{MissionDag, MissionNode};

/// Quality threshold below which autonomous replanning is triggered.
const QUALITY_THRESHOLD: f64 = 0.6;

/// Maximum retry ratio before switching to a different strategy.
const MAX_RETRY_RATIO: f64 = 0.5;

/// Autonomous replanner that analyses evaluation scores and mission reflections
/// to decide whether and how to replan.
pub struct AutonomousReplanner;

impl Default for AutonomousReplanner {
    fn default() -> Self {
        Self::new()
    }
}

impl AutonomousReplanner {
    pub fn new() -> Self {
        Self
    }

    /// Determine whether a mission needs replanning based on its score.
    pub fn should_replan(&self, score: &MissionScore) -> bool {
        score.overall_score < QUALITY_THRESHOLD
    }

    /// Full evaluation + replanning pipeline.
    pub fn evaluate_and_replan(
        &self,
        _mission_id: MissionId,
        dag: &MissionDag,
        score: &MissionScore,
        reflection: &MissionReflection,
    ) -> Result<Vec<ReplanningAction>, String> {
        if !self.should_replan(score) {
            return Ok(Vec::new());
        }

        let mut actions = Vec::new();

        // Check for task-level failures
        if reflection.failed_tasks > 0 {
            let trigger =
                ReplanningTrigger::QualityBelowThreshold((score.overall_score * 100.0) as u64);
            actions.extend(self.determine_actions(trigger, dag, reflection));
        }

        // Check for high retry ratio
        if reflection.retry_ratio() > MAX_RETRY_RATIO {
            actions.push(ReplanningAction::Escalate(
                "High retry ratio — consider strategy change".to_string(),
            ));
        }

        // Check for agent failures and recommend replacements
        for (agent_id, effectiveness) in &reflection.agent_effectiveness {
            if effectiveness.success_rate() < 0.5 && effectiveness.task_count > 1 {
                let new_agent = AgentId::new();
                actions.push(ReplanningAction::ReplaceAgent(*agent_id, new_agent));
            }
        }

        // Check for budget exhaustion
        if reflection.total_cost > 90.0 {
            actions.push(ReplanningAction::Escalate(
                "Budget nearly exhausted".to_string(),
            ));
        }

        // If nothing specific, rebuild DAG
        if actions.is_empty() {
            actions.push(ReplanningAction::RebuildDag(dag.clone()));
        }

        Ok(actions)
    }

    /// Map a trigger to concrete replanning actions.
    pub fn determine_actions(
        &self,
        trigger: ReplanningTrigger,
        dag: &MissionDag,
        reflection: &MissionReflection,
    ) -> Vec<ReplanningAction> {
        match trigger {
            ReplanningTrigger::TaskFailure(task_id) => {
                // If the task has been retried too many times, split it
                if reflection.retries > 3 {
                    let sub_nodes = self.split_task_heuristic(&task_id);
                    vec![ReplanningAction::SplitTask(task_id, sub_nodes)]
                } else {
                    vec![ReplanningAction::RetryStrategy(task_id)]
                }
            }
            ReplanningTrigger::AgentFailure(agent_id) => {
                let new_agent = AgentId::new();
                vec![ReplanningAction::ReplaceAgent(agent_id, new_agent)]
            }
            ReplanningTrigger::BudgetExhaustion => {
                vec![ReplanningAction::Escalate("Budget exhausted".into())]
            }
            ReplanningTrigger::Timeout => {
                vec![ReplanningAction::Escalate("Mission timed out".into())]
            }
            ReplanningTrigger::QualityBelowThreshold(_) => {
                // Try rebuilding the DAG
                vec![ReplanningAction::RebuildDag(dag.clone())]
            }
            ReplanningTrigger::ToolFailure(tool) => {
                vec![ReplanningAction::Escalate(format!("Tool {} failed", tool))]
            }
            ReplanningTrigger::LowConfidence | ReplanningTrigger::MissingInformation => {
                vec![ReplanningAction::ModifyPlan(dag.clone())]
            }
        }
    }

    /// Heuristic: split a failed task into two simpler sub-tasks.
    fn split_task_heuristic(&self, task_id: &TaskId) -> Vec<MissionNode> {
        vec![
            MissionNode {
                id: TaskId::new(),
                name: format!("Sub-task A of {:?}", task_id),
                role: AgentRole::Coder,
                payload: "First half of the original task".into(),
            },
            MissionNode {
                id: TaskId::new(),
                name: format!("Sub-task B of {:?}", task_id),
                role: AgentRole::Coder,
                payload: "Second half of the original task".into(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::models::grade_from_score;
    use crate::reflection::mission_reflection::AgentEffectivenessScore;
    use std::collections::HashMap;

    fn make_score(overall: f64) -> MissionScore {
        MissionScore {
            mission_id: MissionId::new(),
            overall_score: overall,
            metric_scores: vec![],
            evaluated_at: chrono::Utc::now(),
            grade: grade_from_score(overall),
        }
    }

    fn make_reflection(completed: u32, failed: u32, retries: u32, cost: f64) -> MissionReflection {
        MissionReflection {
            mission_id: MissionId::new(),
            total_tasks: completed + failed,
            completed_tasks: completed,
            failed_tasks: failed,
            retries,
            tool_usage: HashMap::new(),
            agent_effectiveness: HashMap::new(),
            total_cost: cost,
            total_latency_ms: 1000,
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
        }
    }

    #[test]
    fn should_replan_below_threshold() {
        let rp = AutonomousReplanner::new();
        assert!(rp.should_replan(&make_score(0.3)));
        assert!(rp.should_replan(&make_score(0.59)));
    }

    #[test]
    fn should_not_replan_above_threshold() {
        let rp = AutonomousReplanner::new();
        assert!(!rp.should_replan(&make_score(0.6)));
        assert!(!rp.should_replan(&make_score(0.9)));
    }

    #[test]
    fn evaluate_good_mission_no_actions() {
        let rp = AutonomousReplanner::new();
        let dag = MissionDag::new();
        let score = make_score(0.85);
        let reflection = make_reflection(10, 0, 0, 10.0);

        let actions = rp
            .evaluate_and_replan(MissionId::new(), &dag, &score, &reflection)
            .unwrap();
        assert!(actions.is_empty());
    }

    #[test]
    fn evaluate_poor_mission_produces_actions() {
        let rp = AutonomousReplanner::new();
        let dag = MissionDag::new();
        let score = make_score(0.3);
        let reflection = make_reflection(3, 7, 5, 50.0);

        let actions = rp
            .evaluate_and_replan(MissionId::new(), &dag, &score, &reflection)
            .unwrap();
        assert!(!actions.is_empty());
    }

    #[test]
    fn high_retry_triggers_escalation() {
        let rp = AutonomousReplanner::new();
        let dag = MissionDag::new();
        let score = make_score(0.4);
        let reflection = make_reflection(5, 5, 10, 10.0); // 100% retry ratio

        let actions = rp
            .evaluate_and_replan(MissionId::new(), &dag, &score, &reflection)
            .unwrap();
        let has_escalation = actions
            .iter()
            .any(|a| matches!(a, ReplanningAction::Escalate(_)));
        assert!(has_escalation);
    }

    #[test]
    fn poor_agent_triggers_replacement() {
        let rp = AutonomousReplanner::new();
        let dag = MissionDag::new();
        let score = make_score(0.4);
        let agent = AgentId::new();
        let mut reflection = make_reflection(3, 7, 0, 10.0);
        reflection.agent_effectiveness.insert(
            agent,
            AgentEffectivenessScore {
                tasks_completed: 1,
                tasks_failed: 9,
                total_quality: 2.0,
                total_latency_ms: 10000,
                task_count: 10,
            },
        );

        let actions = rp
            .evaluate_and_replan(MissionId::new(), &dag, &score, &reflection)
            .unwrap();
        let has_replacement = actions
            .iter()
            .any(|a| matches!(a, ReplanningAction::ReplaceAgent(_, _)));
        assert!(has_replacement);
    }

    #[test]
    fn budget_exhaustion_triggers_escalation() {
        let rp = AutonomousReplanner::new();
        let dag = MissionDag::new();
        let score = make_score(0.5);
        let reflection = make_reflection(5, 5, 0, 95.0);

        let actions = rp
            .evaluate_and_replan(MissionId::new(), &dag, &score, &reflection)
            .unwrap();
        let has_budget = actions
            .iter()
            .any(|a| matches!(a, ReplanningAction::Escalate(msg) if msg.contains("Budget")));
        assert!(has_budget);
    }

    #[test]
    fn task_failure_with_many_retries_splits() {
        let rp = AutonomousReplanner::new();
        let dag = MissionDag::new();
        let reflection = make_reflection(3, 7, 5, 10.0);
        let trigger = ReplanningTrigger::TaskFailure(TaskId::new());

        let actions = rp.determine_actions(trigger, &dag, &reflection);
        let has_split = actions
            .iter()
            .any(|a| matches!(a, ReplanningAction::SplitTask(_, _)));
        assert!(has_split);
    }

    #[test]
    fn task_failure_few_retries_retries() {
        let rp = AutonomousReplanner::new();
        let dag = MissionDag::new();
        let reflection = make_reflection(3, 2, 1, 10.0);
        let trigger = ReplanningTrigger::TaskFailure(TaskId::new());

        let actions = rp.determine_actions(trigger, &dag, &reflection);
        let has_retry = actions
            .iter()
            .any(|a| matches!(a, ReplanningAction::RetryStrategy(_)));
        assert!(has_retry);
    }

    #[test]
    fn quality_below_threshold_rebuilds_dag() {
        let rp = AutonomousReplanner::new();
        let dag = MissionDag::new();
        let reflection = make_reflection(5, 5, 0, 10.0);
        let trigger = ReplanningTrigger::QualityBelowThreshold(40);

        let actions = rp.determine_actions(trigger, &dag, &reflection);
        let has_rebuild = actions
            .iter()
            .any(|a| matches!(a, ReplanningAction::RebuildDag(_)));
        assert!(has_rebuild);
    }

    #[test]
    fn timeout_escalates() {
        let rp = AutonomousReplanner::new();
        let dag = MissionDag::new();
        let reflection = make_reflection(5, 0, 0, 0.0);
        let trigger = ReplanningTrigger::Timeout;

        let actions = rp.determine_actions(trigger, &dag, &reflection);
        let has_escalation = actions
            .iter()
            .any(|a| matches!(a, ReplanningAction::Escalate(_)));
        assert!(has_escalation);
    }

    #[test]
    fn split_task_produces_two_subtasks() {
        let rp = AutonomousReplanner::new();
        let task_id = TaskId::new();
        let sub = rp.split_task_heuristic(&task_id);
        assert_eq!(sub.len(), 2);
    }

    #[test]
    fn default_trait() {
        let rp = AutonomousReplanner::new();
        assert!(!rp.should_replan(&make_score(0.9)));
    }
}
