use crate::db::Store;
use ares_core::{AresError, Goal, Plan, Milestone, Task, TaskDependency, PlanDetails, PlanStatus, TaskStatus};
use rusqlite::{params, OptionalExtension};
use chrono::{DateTime, Utc};
use std::str::FromStr;

pub struct SqlitePlanRepository {
    store: Store,
}

impl SqlitePlanRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn create_goal(&self, goal: &Goal) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO goals (id, title, description, priority, deadline, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                goal.id,
                goal.title,
                goal.description,
                goal.priority,
                goal.deadline.map(|d| d.to_rfc3339()),
                goal.created_at.to_rfc3339(),
                goal.updated_at.to_rfc3339(),
            ],
        ).map_err(AresError::db)?;

        // Also initialize goal state to "Planning"
        conn.execute(
            "INSERT OR REPLACE INTO goal_states (goal_id, state, confidence, updated_at)
             VALUES (?1, 'Planning', 1.0, ?2)",
            params![goal.id, Utc::now().to_rfc3339()],
        ).map_err(AresError::db)?;

        Ok(())
    }

    pub fn get_goal(&self, id: &str) -> Result<Option<Goal>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, deadline, created_at, updated_at FROM goals WHERE id = ?1"
        ).map_err(AresError::db)?;

        let row = stmt.query_row(params![id], |r| {
            let id: String = r.get(0)?;
            let title: String = r.get(1)?;
            let description: Option<String> = r.get(2)?;
            let priority: String = r.get(3)?;
            let deadline_str: Option<String> = r.get(4)?;
            let created_str: String = r.get(5)?;
            let updated_str: String = r.get(6)?;

            let deadline = deadline_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok(Goal {
                id,
                title,
                description,
                priority,
                deadline,
                created_at,
                updated_at,
            })
        }).optional().map_err(AresError::db)?;

        Ok(row)
    }

    pub fn create_plan(&self, plan: &Plan) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO plans (id, goal_id, state, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                plan.id,
                plan.goal_id,
                plan.state.to_string(),
                plan.created_at.to_rfc3339(),
                plan.updated_at.to_rfc3339(),
            ],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn get_plan(&self, id: &str) -> Result<Option<Plan>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, goal_id, state, created_at, updated_at FROM plans WHERE id = ?1"
        ).map_err(AresError::db)?;

        let row = stmt.query_row(params![id], |r| {
            let id: String = r.get(0)?;
            let goal_id: String = r.get(1)?;
            let state_str: String = r.get(2)?;
            let created_str: String = r.get(3)?;
            let updated_str: String = r.get(4)?;

            let state = PlanStatus::from_str(&state_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))))?;

            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok(Plan {
                id,
                goal_id,
                state,
                created_at,
                updated_at,
            })
        }).optional().map_err(AresError::db)?;

        Ok(row)
    }

    pub fn create_milestone(&self, milestone: &Milestone) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO milestones (id, plan_id, title, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                milestone.id,
                milestone.plan_id,
                milestone.title,
                milestone.description,
                milestone.created_at.to_rfc3339(),
            ],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn create_task(&self, task: &Task) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO tasks (id, milestone_id, plan_id, title, description, status, estimated_duration, complexity, execution_order, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                task.id,
                task.milestone_id,
                task.plan_id,
                task.title,
                task.description,
                task.status.to_string(),
                task.estimated_duration,
                task.complexity,
                task.execution_order,
                task.created_at.to_rfc3339(),
            ],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn create_task_dependency(&self, dep: &TaskDependency) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO task_dependencies (task_id, depends_on_id)
             VALUES (?1, ?2)",
            params![dep.task_id, dep.depends_on_id],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn get_milestones_for_plan(&self, plan_id: &str) -> Result<Vec<Milestone>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, plan_id, title, description, created_at FROM milestones WHERE plan_id = ?1"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map(params![plan_id], |r| {
            let id: String = r.get(0)?;
            let plan_id: String = r.get(1)?;
            let title: String = r.get(2)?;
            let description: Option<String> = r.get(3)?;
            let created_str: String = r.get(4)?;

            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok(Milestone {
                id,
                plan_id,
                title,
                description,
                created_at,
            })
        }).map_err(AresError::db)?;

        let mut milestones = Vec::new();
        for m in rows {
            milestones.push(m.map_err(AresError::db)?);
        }
        Ok(milestones)
    }

    pub fn get_tasks_for_plan(&self, plan_id: &str) -> Result<Vec<Task>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, milestone_id, plan_id, title, description, status, estimated_duration, complexity, execution_order, created_at FROM tasks WHERE plan_id = ?1 ORDER BY execution_order ASC"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map(params![plan_id], |r| {
            let id: String = r.get(0)?;
            let milestone_id: Option<String> = r.get(1)?;
            let plan_id: String = r.get(2)?;
            let title: String = r.get(3)?;
            let description: Option<String> = r.get(4)?;
            let status_str: String = r.get(5)?;
            let estimated_duration: Option<i32> = r.get(6)?;
            let complexity: Option<String> = r.get(7)?;
            let execution_order: i32 = r.get(8)?;
            let created_str: String = r.get(9)?;

            let status = TaskStatus::from_str(&status_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))))?;

            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(9, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok(Task {
                id,
                milestone_id,
                plan_id,
                title,
                description,
                status,
                estimated_duration,
                complexity,
                execution_order,
                created_at,
            })
        }).map_err(AresError::db)?;

        let mut tasks = Vec::new();
        for t in rows {
            tasks.push(t.map_err(AresError::db)?);
        }
        Ok(tasks)
    }

    pub fn get_dependencies_for_plan(&self, plan_id: &str) -> Result<Vec<TaskDependency>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT td.task_id, td.depends_on_id 
             FROM task_dependencies td
             JOIN tasks t ON td.task_id = t.id
             WHERE t.plan_id = ?1"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map(params![plan_id], |r| {
            let task_id: String = r.get(0)?;
            let depends_on_id: String = r.get(1)?;
            Ok(TaskDependency {
                task_id,
                depends_on_id,
            })
        }).map_err(AresError::db)?;

        let mut deps = Vec::new();
        for d in rows {
            deps.push(d.map_err(AresError::db)?);
        }
        Ok(deps)
    }

    pub fn update_plan_status(&self, plan_id: &str, status: PlanStatus) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE plans SET state = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.to_string(), Utc::now().to_rfc3339(), plan_id],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            params![status.to_string(), task_id],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn list_plans(&self) -> Result<Vec<Plan>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, goal_id, state, created_at, updated_at FROM plans ORDER BY created_at DESC"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map([], |r| {
            let id: String = r.get(0)?;
            let goal_id: String = r.get(1)?;
            let state_str: String = r.get(2)?;
            let created_str: String = r.get(3)?;
            let updated_str: String = r.get(4)?;

            let state = PlanStatus::from_str(&state_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))))?;

            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok(Plan {
                id,
                goal_id,
                state,
                created_at,
                updated_at,
            })
        }).map_err(AresError::db)?;

        let mut plans = Vec::new();
        for p in rows {
            plans.push(p.map_err(AresError::db)?);
        }
        Ok(plans)
    }

    pub fn get_plan_details(&self, plan_id: &str) -> Result<Option<PlanDetails>, AresError> {
        let plan = match self.get_plan(plan_id)? {
            Some(p) => p,
            None => return Ok(None),
        };
        let goal = match self.get_goal(&plan.goal_id)? {
            Some(g) => g,
            None => return Err(AresError::not_found("goal", &plan.goal_id)),
        };
        let milestones = self.get_milestones_for_plan(plan_id)?;
        let tasks = self.get_tasks_for_plan(plan_id)?;
        let dependencies = self.get_dependencies_for_plan(plan_id)?;

        Ok(Some(PlanDetails {
            plan,
            goal,
            milestones,
            tasks,
            dependencies,
        }))
    }
}
