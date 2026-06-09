use crate::events::models::GoalCreatedPayload;
use crate::events::publisher::PlannerEventPublisher;
use crate::goals::state_machine::GoalStateMachine;
use crate::models::goal::{Goal, GoalPriority, GoalState, GoalStateRecord, GoalStateTransition};
use crate::repository::goals::SqliteGoalRepository;
use ares_core::id::GoalId;
use ares_core::{AresError, ProjectId};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct GoalService {
    repo: Arc<SqliteGoalRepository>,
    publisher: Arc<PlannerEventPublisher>,
}

impl GoalService {
    pub fn new(repo: Arc<SqliteGoalRepository>, publisher: Arc<PlannerEventPublisher>) -> Self {
        Self { repo, publisher }
    }

    pub fn create_goal(
        &self,
        project_id: Option<ProjectId>,
        title: String,
        description: Option<String>,
        priority: GoalPriority,
        deadline: Option<chrono::DateTime<Utc>>,
    ) -> Result<Goal, AresError> {
        let now = Utc::now();
        let goal = Goal {
            id: GoalId::new(),
            title,
            description,
            priority,
            deadline,
            created_at: now,
            updated_at: now,
        };

        // 1. Save to repository (enforcing Rule 2)
        self.repo.create(&goal)?;

        // 2. Initial state
        let _initial_state = GoalStateRecord {
            goal_id: goal.id.clone(),
            state: GoalState::Draft,
            confidence: None,
            updated_at: now,
        };
        // TODO: Save state record to a repository

        // 3. Publish event (enforcing Rule 4)
        self.publisher
            .publish_goal_created(project_id, GoalCreatedPayload { goal: goal.clone() })?;

        Ok(goal)
    }

    pub fn transition_goal_state(
        &self,
        goal_id: &GoalId,
        current_state: &GoalState,
        next_state: GoalState,
        reason: Option<String>,
    ) -> Result<GoalStateTransition, AresError> {
        // Enforce state machine rules
        let validated_next_state = GoalStateMachine::transition(current_state, next_state)?;

        let transition = GoalStateTransition {
            id: Uuid::now_v7().to_string(),
            goal_id: goal_id.clone(),
            from_state: Some(current_state.clone()),
            to_state: validated_next_state.clone(),
            reason,
            transitioned_at: Utc::now(),
        };

        // TODO: Persist transition to repository

        Ok(transition)
    }
}
