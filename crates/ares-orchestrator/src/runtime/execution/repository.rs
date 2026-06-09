use super::models::{DistributedExecution, DistributedExecutionAttempt, WorkflowExecutionStep};
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

pub struct ExecutionRepository {
    store: Store,
}

impl ExecutionRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn insert_execution(&self, exec: &DistributedExecution) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO distributed_executions (id, workflow_id, status, created_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                exec.id,
                exec.workflow_id,
                exec.status,
                exec.created_at,
                exec.completed_at
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    pub fn insert_attempt(&self, attempt: &DistributedExecutionAttempt) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO distributed_execution_attempts (id, execution_id, worker_id, lease_id, attempt_number, assigned_at, started_at, completed_at, execution_duration_ms, execution_node, status, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                attempt.id,
                attempt.execution_id,
                attempt.worker_id,
                attempt.lease_id,
                attempt.attempt_number,
                attempt.assigned_at,
                attempt.started_at,
                attempt.completed_at,
                attempt.execution_duration_ms,
                attempt.execution_node,
                attempt.status,
                attempt.error_message
            ],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn insert_step(&self, step: &WorkflowExecutionStep) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO workflow_execution_steps (id, attempt_id, step_name, status, started_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![step.id, step.attempt_id, step.step_name, step.status, step.started_at, step.completed_at],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn list_executions(&self, limit: usize) -> Result<Vec<DistributedExecution>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, workflow_id, status, created_at, completed_at FROM distributed_executions ORDER BY created_at DESC LIMIT ?1"
        ).map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(DistributedExecution {
                    id: row.get(0)?,
                    workflow_id: row.get(1)?,
                    status: row.get(2)?,
                    created_at: row.get(3)?,
                    completed_at: row.get(4)?,
                })
            })
            .map_err(AresError::db)?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r.map_err(AresError::db)?);
        }
        Ok(items)
    }
}
