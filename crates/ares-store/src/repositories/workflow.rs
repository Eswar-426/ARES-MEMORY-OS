//! Workflow repository — SQLite persistence for workflows, versions,
//! executions, events, snapshots, and dead letters.
//!
//! All state transitions are transactional: event + state are committed atomically.

use crate::db::Store;
use ares_core::types::event::now_micros;
use ares_core::{
    AresError, DeadLetterEntry, EventId, ExecutionId, StepId, WorkflowEvent, WorkflowEventType,
    WorkflowExecutionSnapshot, WorkflowId, WorkflowStatus,
};
use rusqlite::params;
use tracing::debug;

pub struct SqliteWorkflowRepository {
    store: Store,
}

impl SqliteWorkflowRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl super::traits::WorkflowRepository for SqliteWorkflowRepository {
    // ─────────────────────────────────────────────────────────────
    // Workflow CRUD
    // ─────────────────────────────────────────────────────────────

    /// Create a new workflow entry.
    fn create_workflow(
        &self,
        id: &WorkflowId,
        name: &str,
        description: &str,
    ) -> Result<(), AresError> {
        let now = now_micros();
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO workflows (id, name, description, current_version, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 1, 'active', ?4, ?5)",
            params![id.as_str(), name, description, now, now],
        )
        .map_err(AresError::db)?;
        debug!(workflow_id = %id, "Workflow created");
        Ok(())
    }

    /// Create a new immutable workflow version. Returns the version ID.
    fn create_version(
        &self,
        version_id: &str,
        workflow_id: &WorkflowId,
        version: u32,
        definition_json: &str,
        timeout_ms: Option<u64>,
    ) -> Result<(), AresError> {
        let now = now_micros();
        self.store.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO workflow_versions (id, workflow_id, version, definition_json, timeout_ms, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![version_id, workflow_id.as_str(), version, definition_json, timeout_ms.map(|v| v as i64), now],
            )
            .map_err(AresError::db)?;

            tx.execute(
                "UPDATE workflows SET current_version = ?1, updated_at = ?2 WHERE id = ?3",
                params![version, now, workflow_id.as_str()],
            )
            .map_err(AresError::db)?;

            Ok(())
        })?;
        debug!(workflow_id = %workflow_id, version = version, "Workflow version created");
        Ok(())
    }

    /// Load a workflow definition JSON by version ID.
    fn get_version_definition(&self, version_id: &str) -> Result<String, AresError> {
        let conn = self.store.get_conn()?;
        conn.query_row(
            "SELECT definition_json FROM workflow_versions WHERE id = ?1",
            params![version_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AresError::not_found("workflow_version", version_id)
            }
            other => AresError::db(other),
        })
    }

    // ─────────────────────────────────────────────────────────────
    // Execution CRUD
    // ─────────────────────────────────────────────────────────────

    /// Create a new execution record.
    fn create_execution(
        &self,
        execution_id: &ExecutionId,
        workflow_version_id: &str,
    ) -> Result<(), AresError> {
        let now = now_micros();
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO workflow_executions (id, workflow_version_id, status, created_at)
             VALUES (?1, ?2, 'pending', ?3)",
            params![execution_id.as_str(), workflow_version_id, now],
        )
        .map_err(AresError::db)?;
        debug!(execution_id = %execution_id, "Execution created");
        Ok(())
    }

    /// Atomically create execution and insert initial events.
    fn start_workflow_execution(
        &self,
        execution_id: &ExecutionId,
        workflow_version_id: &str,
        events: Vec<WorkflowEvent>,
        status: &WorkflowStatus,
    ) -> Result<(), AresError> {
        let now = now_micros();
        self.store.with_transaction(|tx| {
            // Create execution
            tx.execute(
                "INSERT INTO workflow_executions (id, workflow_version_id, status, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![execution_id.as_str(), workflow_version_id, status.as_str(), now],
            )
            .map_err(AresError::db)?;

            let expected_version = events.len() as i64;
            // Insert events
            for event in events {
                tx.execute(
                    "INSERT INTO workflow_events (id, execution_id, step_id, sequence_number, schema_version, event_type, payload_json, ts)
                     VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        event.id.as_str(),
                        event.execution_id.as_str(),
                        event.sequence_number as i64,
                        event.schema_version,
                        event.event_type.as_str(),
                        event.payload,
                        event.timestamp,
                    ],
                )
                .map_err(AresError::db)?;
            }

            // Update execution start_ts and version
            tx.execute(
                "UPDATE workflow_executions SET start_ts = ?1, version = ?2 WHERE id = ?3",
                params![now, expected_version, execution_id.as_str()],
            )
            .map_err(AresError::db)?;

            Ok(())
        })?;
        debug!(execution_id = %execution_id, "Execution started transactionally");
        Ok(())
    }

    /// Atomically append an event and update execution status.
    /// This is the core transactional write pattern.
    fn append_event_and_update_status(
        &self,
        event: &WorkflowEvent,
        new_status: &WorkflowStatus,
        expected_version: u64,
    ) -> Result<(), AresError> {
        self.store.with_transaction(|tx| {
            // Append event
            tx.execute(
                "INSERT INTO workflow_events (id, execution_id, step_id, sequence_number, schema_version, event_type, payload_json, ts)
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7)",
                params![
                    event.id.as_str(),
                    event.execution_id.as_str(),
                    event.sequence_number as i64,
                    event.schema_version,
                    event.event_type.as_str(),
                    event.payload,
                    event.timestamp,
                ],
            )
            .map_err(AresError::db)?;

            // Update execution status and version
            let now = now_micros();
            let rows_affected = tx.execute(
                "UPDATE workflow_executions SET status = ?1, start_ts = COALESCE(start_ts, ?2), version = version + 1 WHERE id = ?3 AND version = ?4",
                params![new_status.as_str(), now, event.execution_id.as_str(), expected_version],
            )
            .map_err(AresError::db)?;

            if rows_affected == 0 {
                return Err(AresError::validation(format!(
                    "Optimistic concurrency control failed for execution {}",
                    event.execution_id
                )));
            }

            Ok(())
        })
    }

    /// Atomically append a step-level event and update execution status.
    fn append_step_event_and_update_status(
        &self,
        event: &WorkflowEvent,
        new_status: &WorkflowStatus,
        step_id: &StepId,
        expected_version: u64,
    ) -> Result<(), AresError> {
        self.store.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO workflow_events (id, execution_id, step_id, sequence_number, schema_version, event_type, payload_json, ts)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    event.id.as_str(),
                    event.execution_id.as_str(),
                    step_id.as_str(),
                    event.sequence_number as i64,
                    event.schema_version,
                    event.event_type.as_str(),
                    event.payload,
                    event.timestamp,
                ],
            )
            .map_err(AresError::db)?;

            let now = now_micros();
            let rows_affected = tx.execute(
                "UPDATE workflow_executions SET status = ?1, start_ts = COALESCE(start_ts, ?2), version = version + 1 WHERE id = ?3 AND version = ?4",
                params![new_status.as_str(), now, event.execution_id.as_str(), expected_version],
            )
            .map_err(AresError::db)?;

            if rows_affected == 0 {
                return Err(AresError::validation(format!(
                    "Optimistic concurrency control failed for execution {}",
                    event.execution_id
                )));
            }

            Ok(())
        })
    }

    /// Get the next sequence number for an execution.
    fn next_sequence_number(&self, execution_id: &ExecutionId) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let max: Option<i64> = conn
            .query_row(
                "SELECT MAX(sequence_number) FROM workflow_events WHERE execution_id = ?1",
                params![execution_id.as_str()],
                |row| row.get(0),
            )
            .map_err(AresError::db)?;
        Ok(max.map(|v| v as u64 + 1).unwrap_or(1))
    }

    // ─────────────────────────────────────────────────────────────
    // Event replay
    // ─────────────────────────────────────────────────────────────

    /// List events for an execution after a given sequence number.
    /// Ordered by sequence_number ASC. Capped at `limit`.
    fn list_events_after(
        &self,
        execution_id: &ExecutionId,
        after_sequence: u64,
    ) -> Result<Vec<WorkflowEvent>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, execution_id, sequence_number, schema_version, event_type, payload_json, ts
                 FROM workflow_events
                 WHERE execution_id = ?1 AND sequence_number > ?2
                 ORDER BY sequence_number ASC
                 LIMIT 10000",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(
                params![execution_id.as_str(), after_sequence as i64],
                |row| {
                    let event_type_str: String = row.get(4)?;
                    Ok(WorkflowEvent {
                        id: EventId::from(row.get::<_, String>(0)?),
                        execution_id: ExecutionId::from(row.get::<_, String>(1)?),
                        sequence_number: row.get::<_, i64>(2)? as u64,
                        schema_version: row.get::<_, i32>(3)? as u32,
                        event_type: event_type_str
                            .parse()
                            .unwrap_or(WorkflowEventType::WorkflowCreated),
                        payload: row.get(5)?,
                        timestamp: row.get(6)?,
                    })
                },
            )
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    /// Count total events for an execution.
    fn count_events(&self, execution_id: &ExecutionId) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM workflow_events WHERE execution_id = ?1",
                params![execution_id.as_str()],
                |row| row.get(0),
            )
            .map_err(AresError::db)?;
        Ok(count as u64)
    }

    // ─────────────────────────────────────────────────────────────
    // Snapshots
    // ─────────────────────────────────────────────────────────────

    /// Save an execution snapshot (overwrites previous).
    fn save_snapshot(&self, snapshot: &WorkflowExecutionSnapshot) -> Result<(), AresError> {
        let snapshot_json = serde_json::to_string(snapshot)?;
        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE workflow_executions SET snapshot_json = ?1 WHERE id = ?2",
            params![snapshot_json, snapshot.execution_id.as_str()],
        )
        .map_err(AresError::db)?;
        debug!(
            execution_id = %snapshot.execution_id,
            seq = snapshot.last_event_sequence,
            "Snapshot saved"
        );
        Ok(())
    }

    /// Load the latest snapshot for an execution.
    fn load_snapshot(
        &self,
        execution_id: &ExecutionId,
    ) -> Result<Option<WorkflowExecutionSnapshot>, AresError> {
        let conn = self.store.get_conn()?;
        let json: Option<String> = conn
            .query_row(
                "SELECT snapshot_json FROM workflow_executions WHERE id = ?1",
                params![execution_id.as_str()],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AresError::not_found("execution", execution_id.as_str())
                }
                other => AresError::db(other),
            })?;
        match json {
            Some(s) => {
                let snap: WorkflowExecutionSnapshot = serde_json::from_str(&s)?;
                Ok(Some(snap))
            }
            None => Ok(None),
        }
    }

    /// Search and filter executions.
    fn search_executions(
        &self,
        req: &ares_core::types::workflow_api::ExecutionSearchRequest,
    ) -> Result<(Vec<ares_core::types::workflow_api::ExecutionSummary>, u64), AresError> {
        let mut query = "SELECT id, workflow_version_id, status, start_ts, COALESCE(start_ts, 0) as end_ts FROM workflow_executions WHERE 1=1".to_string();
        let mut count_query = "SELECT COUNT(*) FROM workflow_executions WHERE 1=1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];
        let mut param_idx = 1;

        if let Some(workflow_id) = &req.workflow_id {
            // Need a JOIN with versions to filter by root workflow_id.
            query = "SELECT e.id, e.workflow_version_id, e.status, e.start_ts, COALESCE(e.start_ts, 0) as end_ts FROM workflow_executions e JOIN workflow_versions v ON e.workflow_version_id = v.id WHERE 1=1".to_string();
            count_query = "SELECT COUNT(*) FROM workflow_executions e JOIN workflow_versions v ON e.workflow_version_id = v.id WHERE 1=1".to_string();

            let condition = format!(" AND v.workflow_id = ?{}", param_idx);
            query.push_str(&condition);
            count_query.push_str(&condition);
            params_vec.push(Box::new(workflow_id.clone()));
            param_idx += 1;
        }

        if let Some(status) = &req.status {
            let condition = format!(" AND e.status = ?{}", param_idx);
            query.push_str(&condition);
            count_query.push_str(&condition);
            params_vec.push(Box::new(status.clone()));
            param_idx += 1;
        }

        if let Some(start_time) = req.start_time {
            let condition = format!(" AND e.start_ts >= ?{}", param_idx);
            query.push_str(&condition);
            count_query.push_str(&condition);
            params_vec.push(Box::new(start_time));
            param_idx += 1;
        }

        if let Some(end_time) = req.end_time {
            let condition = format!(" AND e.start_ts <= ?{}", param_idx);
            query.push_str(&condition);
            count_query.push_str(&condition);
            params_vec.push(Box::new(end_time));
        }

        // Add order and pagination
        query.push_str(" ORDER BY start_ts DESC");

        let page = req.page.unwrap_or(1);
        let page_size = req.page_size.unwrap_or(50);
        let offset = (page.saturating_sub(1)) * page_size;

        query.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

        let conn = self.store.get_conn()?;

        // Count total
        let mut count_stmt = conn.prepare(&count_query).map_err(AresError::db)?;
        let sqlite_params: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let total: u64 = count_stmt
            .query_row(rusqlite::params_from_iter(sqlite_params.iter()), |row| {
                row.get(0)
            })
            .map_err(AresError::db)?;

        // Query data
        let mut stmt = conn.prepare(&query).map_err(AresError::db)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(sqlite_params.iter()), |row| {
                let status_str: String = row.get(2)?;
                let status = serde_json::from_value(serde_json::json!(status_str))
                    .unwrap_or(WorkflowStatus::Failed);

                Ok(ares_core::types::workflow_api::ExecutionSummary {
                    id: ExecutionId(row.get(0)?),
                    workflow_version_id: row.get(1)?,
                    status,
                    start_ts: row.get(3)?,
                    end_ts: row.get(4)?,
                })
            })
            .map_err(AresError::db)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AresError::db)?);
        }

        Ok((results, total))
    }

    // ─────────────────────────────────────────────────────────────
    // Dead letter queue
    // ─────────────────────────────────────────────────────────────

    /// Insert a dead letter entry.
    fn insert_dead_letter(&self, entry: &DeadLetterEntry) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO workflow_dead_letters
             (execution_id, step_id, workflow_version_id, step_name, failure_reason,
              attempt_count, last_error, last_agent_id, execution_duration_ms, failed_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                entry.execution_id.as_str(),
                entry.step_id.as_str(),
                entry.workflow_version_id,
                entry.step_name,
                entry.failure_reason,
                entry.attempt_count,
                entry.last_error,
                entry.last_agent_id.as_ref().map(|a| a.as_str()),
                entry.execution_duration_ms as i64,
                entry.failed_at,
                entry.created_at,
            ],
        )
        .map_err(AresError::db)?;
        debug!(
            execution_id = %entry.execution_id,
            step_id = %entry.step_id,
            "Dead letter created"
        );
        Ok(())
    }

    /// List dead letters for an execution.
    fn list_dead_letters(&self, limit: u32) -> Result<Vec<DeadLetterEntry>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT execution_id, step_id, workflow_version_id, step_name, failure_reason,
                        attempt_count, last_error, last_agent_id, execution_duration_ms, failed_at, created_at
                 FROM workflow_dead_letters
                 ORDER BY created_at ASC
                 LIMIT ?1",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let agent_str: Option<String> = row.get(7)?;
                Ok(DeadLetterEntry {
                    execution_id: ExecutionId::from(row.get::<_, String>(0)?),
                    step_id: StepId::from(row.get::<_, String>(1)?),
                    workflow_version_id: row.get(2)?,
                    step_name: row.get(3)?,
                    failure_reason: row.get(4)?,
                    attempt_count: row.get::<_, i32>(5)? as u32,
                    last_error: row.get(6)?,
                    last_agent_id: agent_str.map(ares_core::AgentId::from),
                    execution_duration_ms: row.get::<_, i64>(8)? as u64,
                    failed_at: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    // ─────────────────────────────────────────────────────────────
    // Agent registry
    // ─────────────────────────────────────────────────────────────

    /// Register a new agent.
    fn register_agent(
        &self,
        id: &str,
        name: &str,
        capabilities_json: &str,
        health_json: &str,
        performance_json: &str,
    ) -> Result<(), AresError> {
        let now = now_micros();
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO agent_registry (id, name, capabilities_json, health_json, performance_json, registered_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, capabilities_json, health_json, performance_json, now],
        )
        .map_err(AresError::db)?;
        debug!(agent_id = %id, "Agent registered");
        Ok(())
    }

    /// Load all agents.
    fn list_agents(&self) -> Result<Vec<ares_core::AgentInfo>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, capabilities_json, health_json, performance_json, registered_at
                 FROM agent_registry ORDER BY name ASC",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map([], |row| {
                let caps_str: String = row.get(2)?;
                let health_str: Option<String> = row.get(3)?;
                let perf_str: Option<String> = row.get(4)?;
                Ok(ares_core::AgentInfo {
                    id: ares_core::AgentId::from(row.get::<_, String>(0)?),
                    name: row.get(1)?,
                    capabilities: serde_json::from_str(&caps_str).unwrap_or_default(),
                    health: health_str
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    performance: perf_str
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    registered_at: row.get(5)?,
                })
            })
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    /// Update agent health.
    fn update_agent_health(&self, agent_id: &str, health_json: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE agent_registry SET health_json = ?1 WHERE id = ?2",
            params![health_json, agent_id],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Update agent performance.
    fn update_agent_performance(
        &self,
        agent_id: &str,
        performance_json: &str,
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE agent_registry SET performance_json = ?1 WHERE id = ?2",
            params![performance_json, agent_id],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Complete an execution (set end_ts).
    fn complete_execution(
        &self,
        execution_id: &ExecutionId,
        status: &WorkflowStatus,
    ) -> Result<(), AresError> {
        let now = now_micros();
        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE workflow_executions SET status = ?1, end_ts = ?2 WHERE id = ?3",
            params![status.as_str(), now, execution_id.as_str()],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get execution status.
    fn get_execution_status(
        &self,
        execution_id: &ExecutionId,
    ) -> Result<WorkflowStatus, AresError> {
        let conn = self.store.get_conn()?;
        let status_str: String = conn
            .query_row(
                "SELECT status FROM workflow_executions WHERE id = ?1",
                params![execution_id.as_str()],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AresError::not_found("execution", execution_id.as_str())
                }
                other => AresError::db(other),
            })?;
        status_str
            .parse()
            .map_err(|e: String| AresError::validation(e))
    }
    fn update_analytics_cache(&self, _duration_ms: f64, is_success: bool) -> Result<(), AresError> {
        let status_col = if is_success {
            "completed_executions"
        } else {
            "failed_executions"
        };
        let query = format!(
            "UPDATE workflow_analytics_cache SET 
                total_executions = total_executions + 1,
                {} = {} + 1,
                updated_at = ?1
             WHERE id = 1",
            status_col, status_col
        );
        let conn = self.store.get_conn()?;
        conn.execute(&query, params![now_micros()])
            .map_err(AresError::db)?;
        Ok(())
    }

    fn get_analytics_cache(&self) -> Result<(u64, u64, u64), AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare("SELECT total_executions, completed_executions, failed_executions FROM workflow_analytics_cache WHERE id = 1").map_err(AresError::db)?;
        let mut rows = stmt.query([]).map_err(AresError::db)?;
        if let Some(row) = rows.next().map_err(AresError::db)? {
            let total: i64 = row.get(0).map_err(AresError::db)?;
            let comp: i64 = row.get(1).map_err(AresError::db)?;
            let fail: i64 = row.get(2).map_err(AresError::db)?;
            return Ok((total as u64, comp as u64, fail as u64));
        }
        Ok((0, 0, 0))
    }

    fn get_visualization(&self, version_id: &str) -> Result<Option<String>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT graph_json FROM workflow_visualizations WHERE workflow_version_id = ?1",
            )
            .map_err(AresError::db)?;

        let mut rows = stmt.query(params![version_id]).map_err(AresError::db)?;
        if let Some(row) = rows.next().map_err(AresError::db)? {
            let graph_json: String = row.get(0).map_err(AresError::db)?;
            return Ok(Some(graph_json));
        }
        Ok(None)
    }

    fn save_visualization(&self, version_id: &str, graph_json: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO workflow_visualizations (workflow_version_id, graph_json, generated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(workflow_version_id) DO UPDATE SET
                graph_json=excluded.graph_json,
                generated_at=excluded.generated_at",
            params![version_id, graph_json, now_micros()],
        )
        .map_err(AresError::db)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_helpers::test_store;
    use crate::repositories::traits::WorkflowRepository;
    use ares_core::WORKFLOW_EVENT_SCHEMA_VERSION;

    #[test]
    fn create_workflow_and_version() {
        let (store, _dir) = test_store();
        let repo = SqliteWorkflowRepository::new(store);
        let wf_id = WorkflowId::new();
        repo.create_workflow(&wf_id, "test-workflow", "A test")
            .unwrap();
        repo.create_version("v1-id", &wf_id, 1, r#"{"steps":[]}"#, Some(30000))
            .unwrap();
        let def = repo.get_version_definition("v1-id").unwrap();
        assert!(def.contains("steps"));
    }

    #[test]
    fn append_event_and_replay() {
        let (store, _dir) = test_store();
        let repo = SqliteWorkflowRepository::new(store);
        let wf_id = WorkflowId::new();
        let exec_id = ExecutionId::new();

        repo.create_workflow(&wf_id, "test", "").unwrap();
        repo.create_version("v1", &wf_id, 1, "{}", None).unwrap();
        repo.create_execution(&exec_id, "v1").unwrap();

        let event = WorkflowEvent {
            id: EventId::new(),
            execution_id: exec_id.clone(),
            sequence_number: 1,
            schema_version: WORKFLOW_EVENT_SCHEMA_VERSION,
            event_type: WorkflowEventType::WorkflowStarted,
            timestamp: now_micros(),
            payload: "{}".into(),
        };
        repo.append_event_and_update_status(&event, &WorkflowStatus::Running, 0)
            .unwrap();

        let events = repo.list_events_after(&exec_id, 0).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence_number, 1);
        assert_eq!(events[0].event_type, WorkflowEventType::WorkflowStarted);

        let status = repo.get_execution_status(&exec_id).unwrap();
        assert_eq!(status, WorkflowStatus::Running);
    }

    #[test]
    fn snapshot_save_and_load() {
        let (store, _dir) = test_store();
        let repo = SqliteWorkflowRepository::new(store);
        let wf_id = WorkflowId::new();
        let exec_id = ExecutionId::new();

        repo.create_workflow(&wf_id, "test", "").unwrap();
        repo.create_version("v1", &wf_id, 1, "{}", None).unwrap();
        repo.create_execution(&exec_id, "v1").unwrap();

        let snapshot = WorkflowExecutionSnapshot {
            execution_id: exec_id.clone(),
            last_event_sequence: 42,
            created_at: now_micros(),
            checksum: "abc123".into(),
            state_json: r#"{"status":"running"}"#.into(),
        };
        repo.save_snapshot(&snapshot).unwrap();

        let loaded = repo.load_snapshot(&exec_id).unwrap().unwrap();
        assert_eq!(loaded.last_event_sequence, 42);
        assert_eq!(loaded.checksum, "abc123");
    }

    #[test]
    fn dead_letter_insert_and_list() {
        let (store, _dir) = test_store();
        let repo = SqliteWorkflowRepository::new(store);
        let wf_id = WorkflowId::new();
        let exec_id = ExecutionId::new();

        repo.create_workflow(&wf_id, "test", "").unwrap();
        repo.create_version("v1", &wf_id, 1, "{}", None).unwrap();
        repo.create_execution(&exec_id, "v1").unwrap();

        let entry = DeadLetterEntry {
            execution_id: exec_id.clone(),
            step_id: StepId::new(),
            workflow_version_id: "v1".into(),
            step_name: "step-a".into(),
            failure_reason: "timeout".into(),
            attempt_count: 3,
            last_error: "connection refused".into(),
            last_agent_id: None,
            execution_duration_ms: 5000,
            failed_at: now_micros(),
            created_at: now_micros(),
        };
        repo.insert_dead_letter(&entry).unwrap();

        let letters = repo.list_dead_letters(100).unwrap();
        assert_eq!(letters.len(), 1);
        assert_eq!(letters[0].step_name, "step-a");
        assert_eq!(letters[0].attempt_count, 3);
    }

    #[test]
    fn workflow_events_are_append_only() {
        let (store, _dir) = test_store();
        let repo = SqliteWorkflowRepository::new(store.clone());
        let wf_id = WorkflowId::new();
        let exec_id = ExecutionId::new();

        repo.create_workflow(&wf_id, "test", "").unwrap();
        repo.create_version("v1", &wf_id, 1, "{}", None).unwrap();
        repo.create_execution(&exec_id, "v1").unwrap();

        let event = WorkflowEvent {
            id: EventId::new(),
            execution_id: exec_id.clone(),
            sequence_number: 1,
            schema_version: WORKFLOW_EVENT_SCHEMA_VERSION,
            event_type: WorkflowEventType::WorkflowStarted,
            timestamp: now_micros(),
            payload: "{}".into(),
        };
        repo.append_event_and_update_status(&event, &WorkflowStatus::Running, 0)
            .unwrap();

        // Attempt DELETE — should be rejected by trigger
        let conn = store.get_conn().unwrap();
        let result = conn.execute(
            "DELETE FROM workflow_events WHERE id = ?1",
            params![event.id.as_str()],
        );
        assert!(
            result.is_err(),
            "DELETE on workflow_events should be rejected"
        );
    }

    #[test]
    fn agent_register_and_list() {
        let (store, _dir) = test_store();
        let repo = SqliteWorkflowRepository::new(store);
        let agent_id = ares_core::AgentId::new();

        repo.register_agent(
            agent_id.as_str(),
            "test-agent",
            r#"["build","test"]"#,
            r#"{"health_score":1.0,"is_available":true,"last_check":0,"consecutive_failures":0}"#,
            r#"{"total_tasks":0,"successful_tasks":0,"failed_tasks":0,"avg_latency_ms":0.0,"success_rate":1.0}"#,
        )
        .unwrap();

        let agents = repo.list_agents().unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "test-agent");
        assert_eq!(agents[0].capabilities, vec!["build", "test"]);
    }

    #[test]
    fn sequence_number_increments() {
        let (store, _dir) = test_store();
        let repo = SqliteWorkflowRepository::new(store);
        let wf_id = WorkflowId::new();
        let exec_id = ExecutionId::new();

        repo.create_workflow(&wf_id, "test", "").unwrap();
        repo.create_version("v1", &wf_id, 1, "{}", None).unwrap();
        repo.create_execution(&exec_id, "v1").unwrap();

        assert_eq!(repo.next_sequence_number(&exec_id).unwrap(), 1);

        let event = WorkflowEvent {
            id: EventId::new(),
            execution_id: exec_id.clone(),
            sequence_number: 1,
            schema_version: WORKFLOW_EVENT_SCHEMA_VERSION,
            event_type: WorkflowEventType::WorkflowStarted,
            timestamp: now_micros(),
            payload: "{}".into(),
        };
        repo.append_event_and_update_status(&event, &WorkflowStatus::Running, 0)
            .unwrap();

        assert_eq!(repo.next_sequence_number(&exec_id).unwrap(), 2);
    }
}
