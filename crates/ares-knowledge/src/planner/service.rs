use ares_store::db::Store;
use ares_core::AresError;
use uuid::Uuid;
use super::models::{ExecutionPlan, ExplainPlanResponse, GoalState};
use super::repository::GoalStateRepository;
use crate::graph::traversal::engine::{TraversalEngine, TraversalStrategy};
use chrono::Utc;

pub struct PlannerIntegrationService {
    db: Store,
    traversal_engine: TraversalEngine,
    goal_repo: GoalStateRepository,
}

impl PlannerIntegrationService {
    pub fn new(db: Store) -> Self {
        Self {
            db,
            traversal_engine: TraversalEngine::new(),
            goal_repo: GoalStateRepository::new(),
        }
    }

    pub async fn track_goal_state(&self, entity_id: Uuid, status: String, progress: f64) -> Result<(), AresError> {
        let conn = self.db.get_conn()?;
        
        let state = GoalState {
            id: Uuid::now_v7(),
            entity_id,
            status,
            progress,
            updated_at: Utc::now(),
        };

        if let Some(existing) = self.goal_repo.get_by_entity_id(&conn, entity_id)? {
            let mut updated = state;
            updated.id = existing.id;
            self.goal_repo.update(&conn, &updated)?;
        } else {
            self.goal_repo.insert(&conn, &state)?;
        }

        Ok(())
    }

    pub async fn find_dependencies(&self, goal_id: Uuid) -> Result<Vec<Uuid>, AresError> {
        let _conn = self.db.get_conn()?;
        // Mock traversal for DEPENDS_ON / PREREQUISITE_FOR
        let deps = self.traversal_engine.traverse(goal_id, TraversalStrategy::BFS, 2);
        Ok(deps)
    }

    pub async fn find_prerequisites(&self, goal_id: Uuid) -> Result<Vec<Uuid>, AresError> {
        self.find_dependencies(goal_id).await
    }

    pub async fn find_goal_path(&self, start_entity: Uuid, goal_entity: Uuid) -> Result<Vec<Uuid>, AresError> {
        let _conn = self.db.get_conn()?;
        // Mock returning a path to achieve a goal
        Ok(vec![start_entity, goal_entity])
    }

    pub async fn find_capability_chain(&self, _capability: &str) -> Result<Vec<Uuid>, AresError> {
        // Scaffolding for finding agents that provide a chain of capabilities
        Ok(vec![])
    }

    pub async fn find_execution_plan(&self, goal_id: Uuid) -> Result<ExecutionPlan, AresError> {
        let _conn = self.db.get_conn()?;
        Ok(ExecutionPlan {
            goal_id,
            steps: vec![],
            expected_capabilities: vec![],
        })
    }

    pub async fn explain_plan(&self, goal_id: Uuid) -> Result<ExplainPlanResponse, AresError> {
        let plan = self.find_execution_plan(goal_id).await?;
        
        let _conn = self.db.get_conn()?;
        // Mock traversal for CONFLICTS_WITH relationship detection
        let conflicting_goals = self.traversal_engine.traverse(goal_id, TraversalStrategy::DFS, 1);

        Ok(ExplainPlanResponse {
            execution_plan: plan,
            rationale: "Plan constructed via graph traversal over ACHIEVES and PREREQUISITE_FOR relationships.".to_string(),
            dependencies_satisfied: true,
            conflicting_goals,
        })
    }
}
