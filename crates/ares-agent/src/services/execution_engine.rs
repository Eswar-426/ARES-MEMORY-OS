//! Week 8 — Execution Engine: step execution, retry with exponential back-off,
//! verification, compensation (saga pattern), and dead-letter insertion.
//!
//! Failure flow:  StepFailed → Retry → Compensation → DeadLetter
//!
//! Default: 3 retries, exponential back-off starting at 500 ms, max 8 s.

use ares_core::types::event::now_micros;
use ares_core::{
    AgentId, AresError, CompensationAction, DeadLetterEntry, ExecutionState, StepId,
    WorkflowStepDef,
};
use ares_store::repositories::traits::WorkflowRepository;
use std::sync::Arc;

use super::workflow_engine::WorkflowEngine;

// ─────────────────────────────────────────────────────────────────
// Step execution result
// ─────────────────────────────────────────────────────────────────

/// Result of executing a single step.
#[derive(Debug, Clone)]
pub enum StepResult {
    Success,
    Failed { error: String },
    TimedOut,
}

// ─────────────────────────────────────────────────────────────────
// Execution Engine
// ─────────────────────────────────────────────────────────────────

pub struct ExecutionEngine {
    repo: Arc<dyn WorkflowRepository + Send + Sync>,
}

impl ExecutionEngine {
    pub fn new(repo: Arc<dyn WorkflowRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    /// Execute a single workflow step.
    ///
    /// This is a deterministic, rule-based execution that:
    /// 1. Marks the step as started
    /// 2. Runs the step logic (simulated — actual execution is pluggable)
    /// 3. Handles success/failure outcomes
    ///
    /// Returns the step result.
    pub fn execute_step(
        &self,
        engine: &WorkflowEngine,
        state: &mut ExecutionState,
        step_def: &WorkflowStepDef,
        agent_id: Option<&AgentId>,
        step_fn: &dyn Fn(&WorkflowStepDef) -> StepResult,
    ) -> Result<StepResult, AresError> {
        let step_id = &step_def.id;
        let start_ts = now_micros();

        // Record step start
        engine.step_started(state, step_id)?;

        // Execute the step
        let result = step_fn(step_def);

        match &result {
            StepResult::Success => {
                engine.step_completed(state, step_id)?;
            }
            StepResult::Failed { error } => {
                engine.step_failed(state, step_id, error)?;

                // Attempt retries
                let retry_result =
                    self.retry_step(engine, state, step_def, agent_id, step_fn, start_ts)?;

                if let StepResult::Failed { error: final_err } = &retry_result {
                    // Compensation
                    self.compensate_step(engine, state, step_def)?;

                    // Dead letter
                    let elapsed_ms = ((now_micros() - start_ts) / 1000).max(0) as u64;
                    self.send_to_dead_letter(state, step_def, final_err, agent_id, elapsed_ms)?;

                    return Ok(retry_result);
                }

                return Ok(retry_result);
            }
            StepResult::TimedOut => {
                engine.step_failed(state, step_id, "Timed out")?;

                // Compensation
                self.compensate_step(engine, state, step_def)?;

                // Dead letter
                let elapsed_ms = ((now_micros() - start_ts) / 1000).max(0) as u64;
                self.send_to_dead_letter(state, step_def, "Step timed out", agent_id, elapsed_ms)?;
            }
        }

        Ok(result)
    }

    /// Retry a failed step with exponential back-off.
    ///
    /// Respects the step's `RetryPolicy`. Returns the final result after
    /// all retries are exhausted or success.
    pub fn retry_step(
        &self,
        engine: &WorkflowEngine,
        state: &mut ExecutionState,
        step_def: &WorkflowStepDef,
        _agent_id: Option<&AgentId>,
        step_fn: &dyn Fn(&WorkflowStepDef) -> StepResult,
        _start_ts: i64,
    ) -> Result<StepResult, AresError> {
        let policy = &step_def.retry_policy;
        let step_id = &step_def.id;

        for attempt in 1..=policy.max_retries {
            // Record retry event
            engine.step_retrying(state, step_id, attempt)?;

            // Compute back-off delay (not actually sleeping — deterministic simulation)
            let _delay = policy.delay_for_attempt(attempt - 1);

            // Re-execute
            let result = step_fn(step_def);

            match &result {
                StepResult::Success => {
                    // Move from retry back to running, then complete
                    engine.step_started(state, step_id)?;
                    engine.step_completed(state, step_id)?;
                    return Ok(StepResult::Success);
                }
                StepResult::Failed { error } => {
                    engine.step_failed(state, step_id, error)?;
                    // Continue to next retry
                }
                StepResult::TimedOut => {
                    engine.step_failed(state, step_id, "Timed out during retry")?;
                    return Ok(StepResult::TimedOut);
                }
            }
        }

        // All retries exhausted
        Ok(StepResult::Failed {
            error: format!(
                "All {} retries exhausted for step '{}'",
                policy.max_retries, step_def.name
            ),
        })
    }

    /// Verify a step's completion (idempotent check).
    pub fn verify_step(&self, state: &ExecutionState, step_id: &StepId) -> bool {
        let task_id = ares_core::TaskId::from(step_id.as_str());
        state.completed_steps.contains(&task_id)
    }

    /// Execute compensation action for a failed step (saga pattern).
    pub fn compensate_step(
        &self,
        engine: &WorkflowEngine,
        state: &mut ExecutionState,
        step_def: &WorkflowStepDef,
    ) -> Result<(), AresError> {
        match &step_def.compensation {
            CompensationAction::None => {
                // No compensation needed
                Ok(())
            }
            CompensationAction::RollbackStep { step_name } => {
                let payload = serde_json::json!({
                    "step_id": step_def.id.as_str(),
                    "compensation": "rollback",
                    "rollback_step": step_name,
                })
                .to_string();

                let seq = self.repo.next_sequence_number(&state.execution_id)?;
                let event = ares_core::WorkflowEvent {
                    id: ares_core::EventId::new(),
                    execution_id: state.execution_id.clone(),
                    sequence_number: seq,
                    schema_version: ares_core::WORKFLOW_EVENT_SCHEMA_VERSION,
                    event_type: ares_core::WorkflowEventType::StepCompensated,
                    timestamp: now_micros(),
                    payload,
                };
                self.repo.append_step_event_and_update_status(
                    &event,
                    &state.current_status,
                    &step_def.id,
                    state.version,
                )?;
                state.version += 1;
                state.last_event_sequence = seq;
                let _ = engine; // suppress unused warning in this branch
                Ok(())
            }
            CompensationAction::CustomHandler {
                handler_name,
                params,
            } => {
                let payload = serde_json::json!({
                    "step_id": step_def.id.as_str(),
                    "compensation": "custom",
                    "handler": handler_name,
                    "params": params,
                })
                .to_string();

                let seq = self.repo.next_sequence_number(&state.execution_id)?;
                let event = ares_core::WorkflowEvent {
                    id: ares_core::EventId::new(),
                    execution_id: state.execution_id.clone(),
                    sequence_number: seq,
                    schema_version: ares_core::WORKFLOW_EVENT_SCHEMA_VERSION,
                    event_type: ares_core::WorkflowEventType::StepCompensated,
                    timestamp: now_micros(),
                    payload,
                };
                self.repo.append_step_event_and_update_status(
                    &event,
                    &state.current_status,
                    &step_def.id,
                    state.version,
                )?;
                state.version += 1;
                state.last_event_sequence = seq;
                let _ = engine;
                Ok(())
            }
            CompensationAction::LogAndContinue => {
                // Just log — no actual compensation
                tracing::warn!(
                    step = %step_def.id,
                    "Step failed, compensation: log and continue"
                );
                Ok(())
            }
        }
    }

    /// Insert a dead letter entry for a permanently failed step.
    fn send_to_dead_letter(
        &self,
        state: &mut ExecutionState,
        step_def: &WorkflowStepDef,
        error: &str,
        agent_id: Option<&AgentId>,
        execution_duration_ms: u64,
    ) -> Result<(), AresError> {
        let now = now_micros();
        let attempt_count = state
            .step_attempts
            .get(step_def.id.as_str())
            .copied()
            .unwrap_or(0)
            + 1; // +1 for the initial attempt

        let entry = DeadLetterEntry {
            execution_id: state.execution_id.clone(),
            step_id: step_def.id.clone(),
            workflow_version_id: state.workflow_version_id.clone(),
            step_name: step_def.name.clone(),
            failure_reason: format!("Step '{}' permanently failed", step_def.name),
            attempt_count,
            last_error: error.to_string(),
            last_agent_id: agent_id.cloned(),
            execution_duration_ms,
            failed_at: now,
            created_at: now,
        };

        self.repo.insert_dead_letter(&entry)?;

        // Emit dead letter event
        let seq = self.repo.next_sequence_number(&state.execution_id)?;
        let payload = serde_json::json!({
            "step_id": step_def.id.as_str(),
            "step_name": step_def.name,
            "error": error,
        })
        .to_string();

        let event = ares_core::WorkflowEvent {
            id: ares_core::EventId::new(),
            execution_id: state.execution_id.clone(),
            sequence_number: seq,
            schema_version: ares_core::WORKFLOW_EVENT_SCHEMA_VERSION,
            event_type: ares_core::WorkflowEventType::DeadLetterCreated,
            timestamp: now_micros(),
            payload,
        };

        self.repo
            .append_step_event_and_update_status(
                &event,
                &state.current_status,
                &step_def.id,
                state.version,
            )
            .map_err(AresError::db)?;
        state.version += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::test_helpers::test_store;
    use ares_core::{RetryPolicy, StepId, TaskId, WorkflowId, WorkflowStepDef};
    use ares_store::SqliteWorkflowRepository;
    use std::sync::Arc;

    fn setup_engine() -> (
        WorkflowEngine,
        ExecutionEngine,
        ExecutionState,
        WorkflowStepDef,
    ) {
        let (store, _dir) = test_store();
        let dir = Box::leak(Box::new(_dir));
        let _ = dir;

        let repo = Arc::new(SqliteWorkflowRepository::new(store.clone()));
        let wf_id = WorkflowId::new();

        repo.create_workflow(&wf_id, "test", "").unwrap();

        let step_def = WorkflowStepDef {
            id: StepId::from("step-a"),
            name: "Step A".into(),
            description: "Test step".into(),
            required_capability: "build".into(),
            depends_on: vec![],
            timeout_ms: Some(5000),
            retry_policy: RetryPolicy {
                max_retries: 2,
                initial_delay_ms: 100,
                max_delay_ms: 1000,
            },
            compensation: CompensationAction::LogAndContinue,
            priority: Default::default(),
        };

        let def = ares_core::WorkflowDefinition {
            workflow_id: wf_id.clone(),
            version: 1,
            name: "test".into(),
            description: "".into(),
            steps: vec![step_def.clone()],
            timeout_ms: None,
        };
        let def_json = serde_json::to_string(&def).unwrap();
        repo.create_version("v1", &wf_id, 1, &def_json, None)
            .unwrap();

        let wf_engine = WorkflowEngine::new(Arc::new(SqliteWorkflowRepository::new(store.clone())));
        let exec_engine = ExecutionEngine::new(Arc::new(SqliteWorkflowRepository::new(store)));

        let plan = wf_engine.create_execution_plan(&wf_id, "v1", 1).unwrap();
        let state = wf_engine.execute_workflow(&plan).unwrap();

        (wf_engine, exec_engine, state, step_def)
    }

    #[test]
    fn execute_step_success() {
        let (wf_engine, exec_engine, mut state, step_def) = setup_engine();

        let result = exec_engine
            .execute_step(&wf_engine, &mut state, &step_def, None, &|_| {
                StepResult::Success
            })
            .unwrap();

        assert!(matches!(result, StepResult::Success));
        assert!(state.completed_steps.contains(&TaskId::from("step-a")));
    }

    #[test]
    fn execute_step_failure_with_retries() {
        let (wf_engine, exec_engine, mut state, step_def) = setup_engine();

        // Always fails
        let result = exec_engine
            .execute_step(&wf_engine, &mut state, &step_def, None, &|_| {
                StepResult::Failed {
                    error: "boom".into(),
                }
            })
            .unwrap();

        assert!(matches!(result, StepResult::Failed { .. }));
        // Should have attempted 1 (initial) + 2 retries = step_failed called multiple times
        assert!(state.failed_steps.contains(&TaskId::from("step-a")));
    }

    #[test]
    fn execute_step_retry_then_succeed() {
        let (wf_engine, exec_engine, mut state, step_def) = setup_engine();
        let call_count = std::cell::Cell::new(0u32);

        let result = exec_engine
            .execute_step(&wf_engine, &mut state, &step_def, None, &|_| {
                let count = call_count.get();
                call_count.set(count + 1);
                if count < 2 {
                    StepResult::Failed {
                        error: "transient".into(),
                    }
                } else {
                    StepResult::Success
                }
            })
            .unwrap();

        assert!(matches!(result, StepResult::Success));
        assert!(state.completed_steps.contains(&TaskId::from("step-a")));
    }

    #[test]
    fn verify_step_works() {
        let (wf_engine, exec_engine, mut state, step_def) = setup_engine();

        assert!(!exec_engine.verify_step(&state, &step_def.id));

        exec_engine
            .execute_step(&wf_engine, &mut state, &step_def, None, &|_| {
                StepResult::Success
            })
            .unwrap();

        assert!(exec_engine.verify_step(&state, &step_def.id));
    }

    #[test]
    fn dead_letter_on_permanent_failure() {
        let (store, _dir) = test_store();
        let dir = Box::leak(Box::new(_dir));
        let _ = dir;

        let repo = Arc::new(SqliteWorkflowRepository::new(store.clone()));
        let wf_id = WorkflowId::new();
        repo.create_workflow(&wf_id, "test", "").unwrap();

        let step_def = WorkflowStepDef {
            id: StepId::from("step-fail"),
            name: "Failing Step".into(),
            description: "Always fails".into(),
            required_capability: "build".into(),
            depends_on: vec![],
            timeout_ms: None,
            retry_policy: RetryPolicy {
                max_retries: 1,
                initial_delay_ms: 100,
                max_delay_ms: 200,
            },
            compensation: CompensationAction::None,
            priority: Default::default(),
        };

        let def = ares_core::WorkflowDefinition {
            workflow_id: wf_id.clone(),
            version: 1,
            name: "test".into(),
            description: "".into(),
            steps: vec![step_def.clone()],
            timeout_ms: None,
        };
        let def_json = serde_json::to_string(&def).unwrap();
        repo.create_version("v1", &wf_id, 1, &def_json, None)
            .unwrap();

        let wf_engine = WorkflowEngine::new(Arc::new(SqliteWorkflowRepository::new(store.clone())));
        let exec_engine =
            ExecutionEngine::new(Arc::new(SqliteWorkflowRepository::new(store.clone())));

        let plan = wf_engine.create_execution_plan(&wf_id, "v1", 1).unwrap();
        let mut state = wf_engine.execute_workflow(&plan).unwrap();

        exec_engine
            .execute_step(&wf_engine, &mut state, &step_def, None, &|_| {
                StepResult::Failed {
                    error: "permanent".into(),
                }
            })
            .unwrap();

        // Verify dead letter was written
        let dl_repo = Arc::new(SqliteWorkflowRepository::new(store));
        let dead_letters = dl_repo.list_dead_letters(100).unwrap();
        assert_eq!(dead_letters.len(), 1);
        assert_eq!(dead_letters[0].step_name, "Failing Step");
    }
}
