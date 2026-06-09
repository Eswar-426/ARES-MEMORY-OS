use crate::decomposition::engine::GoalDecompositionEngine;
use crate::events::models::GoalDecomposedPayload;
use crate::events::publisher::PlannerEventPublisher;
use crate::models::goal::Goal;
use crate::repository::goals::SqliteGoalRepository;
use ares_core::{AresError, ProjectId};
use std::sync::Arc;

pub struct DecompositionService {
    engine: GoalDecompositionEngine,
    repo: Arc<SqliteGoalRepository>,
    publisher: Arc<PlannerEventPublisher>,
}

impl DecompositionService {
    pub fn new(repo: Arc<SqliteGoalRepository>, publisher: Arc<PlannerEventPublisher>) -> Self {
        Self {
            engine: GoalDecompositionEngine::new(),
            repo,
            publisher,
        }
    }

    pub fn decompose_goal(
        &self,
        project_id: Option<ProjectId>,
        parent_goal: &Goal,
    ) -> Result<Vec<Goal>, AresError> {
        let subgoals = self.engine.decompose(parent_goal)?;

        // 1. Save all subgoals to DB
        for goal in &subgoals {
            self.repo.create(goal)?;
        }

        // 2. Publish event
        self.publisher.publish_goal_decomposed(
            project_id,
            GoalDecomposedPayload {
                parent_goal_id: parent_goal.id.clone(),
                child_goals: subgoals.clone(),
            },
        )?;

        Ok(subgoals)
    }
}
