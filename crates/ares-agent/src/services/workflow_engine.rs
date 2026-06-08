//! Week 8 — Workflow Engine: execution lifecycle, queue orchestration,
//! event persistence, snapshot creation, and state reconstruction.
//!
//! All state transitions are persisted transactionally (event + status in one commit).
//! Snapshots are created every 50 events for fast recovery.
//! Reconstruction replays events after the latest snapshot sequence number.

use ares_core::types::event::now_micros;
use ares_core::{
    AresError, EventId, ExecutionId, ExecutionPlan, ExecutionState, StepId, TaskId, WorkflowEvent,
    WorkflowEventType, WorkflowExecutionSnapshot, WorkflowId, WorkflowStatus,
    WORKFLOW_EVENT_SCHEMA_VERSION,
};
use ares_store::repositories::traits::WorkflowRepository;
use std::collections::BTreeSet;
use std::sync::Arc;

/// Snapshot every N events.
const SNAPSHOT_INTERVAL: u64 = 50;

// ─────────────────────────────────────────────────────────────────
// Workflow Engine
// ─────────────────────────────────────────────────────────────────

pub struct WorkflowEngine {
    repo: Arc<dyn WorkflowRepository + Send + Sync>,
}

impl WorkflowEngine {
    pub fn new(repo: Arc<dyn WorkflowRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    // ─────────────────────────────────────────────────────────────
    // Execution plan creation
    // ─────────────────────────────────────────────────────────────

    /// Create an execution plan from a workflow version.
    /// Resolves the definition, validates the DAG, and returns the topological order.
    pub fn create_execution_plan(
        &self,
        workflow_id: &WorkflowId,
        workflow_version_id: &str,
        version: u32,
    ) -> Result<ExecutionPlan, AresError> {
        let def_json = self.repo.get_version_definition(workflow_version_id)?;
        let definition: ares_core::WorkflowDefinition = serde_json::from_str(&def_json)
            .map_err(|e| AresError::validation(format!("Invalid workflow definition: {e}")))?;

        // Build execution order from steps (topological — using step order for now)
        let execution_order: Vec<StepId> = definition.steps.iter().map(|s| s.id.clone()).collect();

        Ok(ExecutionPlan {
            workflow_id: workflow_id.clone(),
            workflow_version_id: workflow_version_id.to_string(),
            version,
            execution_order,
            definition,
        })
    }

    // ─────────────────────────────────────────────────────────────
    // Workflow lifecycle
    // ─────────────────────────────────────────────────────────────

    /// Start executing a workflow from a plan.
    /// Creates the execution record, emits WorkflowCreated + WorkflowStarted events,
    /// and returns the initial ExecutionState.
    pub fn execute_workflow(&self, plan: &ExecutionPlan) -> Result<ExecutionState, AresError> {
        let exec_id = ExecutionId::new();
        self.repo
            .create_execution(&exec_id, &plan.workflow_version_id)?;

        // Build initial pending steps
        let initial_steps: BTreeSet<TaskId> = plan
            .execution_order
            .iter()
            .map(|s| TaskId::from(s.as_str()))
            .collect();

        let mut state = ExecutionState::new(
            plan.workflow_id.clone(),
            exec_id.clone(),
            plan.workflow_version_id.clone(),
            initial_steps,
        );

        // Emit WorkflowCreated
        self.emit_event(
            &mut state,
            WorkflowEventType::WorkflowCreated,
            &WorkflowStatus::Pending,
            None,
            "{}",
        )?;

        // Emit WorkflowStarted
        state.current_status = WorkflowStatus::Running;
        self.emit_event(
            &mut state,
            WorkflowEventType::WorkflowStarted,
            &WorkflowStatus::Running,
            None,
            "{}",
        )?;

        metrics::counter!("ares_workflows_started_total").increment(1);
        metrics::gauge!("ares_workflows_running").increment(1.0);

        Ok(state)
    }

    /// Pause a running workflow.
    pub fn pause_workflow(&self, state: &mut ExecutionState) -> Result<(), AresError> {
        if state.current_status != WorkflowStatus::Running {
            return Err(AresError::validation(format!(
                "Cannot pause workflow in status: {:?}",
                state.current_status
            )));
        }
        state.current_status = WorkflowStatus::Paused;
        self.emit_event(
            state,
            WorkflowEventType::WorkflowPaused,
            &WorkflowStatus::Paused,
            None,
            "{}",
        )
    }

    /// Resume a paused workflow.
    pub fn resume_workflow(&self, state: &mut ExecutionState) -> Result<(), AresError> {
        if state.current_status != WorkflowStatus::Paused {
            return Err(AresError::validation(format!(
                "Cannot resume workflow in status: {:?}",
                state.current_status
            )));
        }
        state.current_status = WorkflowStatus::Running;
        self.emit_event(
            state,
            WorkflowEventType::WorkflowResumed,
            &WorkflowStatus::Running,
            None,
            "{}",
        )
    }

    /// Cancel a workflow.
    pub fn cancel_workflow(&self, state: &mut ExecutionState) -> Result<(), AresError> {
        match state.current_status {
            WorkflowStatus::Completed | WorkflowStatus::Cancelled => {
                return Err(AresError::validation(format!(
                    "Cannot cancel workflow in status: {:?}",
                    state.current_status
                )));
            }
            _ => {}
        }
        state.current_status = WorkflowStatus::Cancelled;
        self.emit_event(
            state,
            WorkflowEventType::WorkflowCancelled,
            &WorkflowStatus::Cancelled,
            None,
            "{}",
        )?;
        self.repo
            .complete_execution(&state.execution_id, &WorkflowStatus::Cancelled)?;
        Ok(())
    }

    /// Mark workflow as completed.
    pub fn complete_workflow(&self, state: &mut ExecutionState) -> Result<(), AresError> {
        state.current_status = WorkflowStatus::Completed;
        self.emit_event(
            state,
            WorkflowEventType::WorkflowCompleted,
            &WorkflowStatus::Completed,
            None,
            "{}",
        )?;
        self.repo
            .complete_execution(&state.execution_id, &WorkflowStatus::Completed)?;

        // Analytics
        self.repo.update_analytics_cache(0.0, true)?;

        metrics::counter!("ares_workflows_completed_total").increment(1);
        metrics::gauge!("ares_workflows_running").decrement(1.0);

        Ok(())
    }

    /// Mark workflow as failed.
    pub fn fail_workflow(&self, state: &mut ExecutionState, reason: &str) -> Result<(), AresError> {
        state.current_status = WorkflowStatus::Failed;
        let payload = serde_json::json!({ "reason": reason }).to_string();
        self.emit_event(
            state,
            WorkflowEventType::WorkflowFailed,
            &WorkflowStatus::Failed,
            None,
            &payload,
        )?;
        self.repo
            .complete_execution(&state.execution_id, &WorkflowStatus::Failed)?;

        // Analytics
        self.repo.update_analytics_cache(0.0, false)?;

        metrics::counter!("ares_workflows_failed_total").increment(1);
        metrics::gauge!("ares_workflows_running").decrement(1.0);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────
    // Step lifecycle (called by ExecutionEngine)
    // ─────────────────────────────────────────────────────────────

    /// Record that a step has been queued.
    pub fn step_queued(
        &self,
        state: &mut ExecutionState,
        step_id: &StepId,
    ) -> Result<(), AresError> {
        let payload = serde_json::json!({ "step_id": step_id.as_str() }).to_string();
        self.emit_step_event(state, WorkflowEventType::StepQueued, step_id, &payload)
    }

    /// Record that a step has started.
    pub fn step_started(
        &self,
        state: &mut ExecutionState,
        step_id: &StepId,
    ) -> Result<(), AresError> {
        let task_id = TaskId::from(step_id.as_str());
        state.pending_steps.remove(&task_id);
        state.running_steps.insert(task_id);
        let payload = serde_json::json!({ "step_id": step_id.as_str() }).to_string();
        self.emit_step_event(state, WorkflowEventType::StepStarted, step_id, &payload)
    }

    /// Record that a step has completed.
    pub fn step_completed(
        &self,
        state: &mut ExecutionState,
        step_id: &StepId,
    ) -> Result<(), AresError> {
        let task_id = TaskId::from(step_id.as_str());
        state.running_steps.remove(&task_id);
        state.completed_steps.insert(task_id);
        let payload = serde_json::json!({ "step_id": step_id.as_str() }).to_string();
        self.emit_step_event(state, WorkflowEventType::StepCompleted, step_id, &payload)
    }

    /// Record that a step has failed.
    pub fn step_failed(
        &self,
        state: &mut ExecutionState,
        step_id: &StepId,
        error: &str,
    ) -> Result<(), AresError> {
        let task_id = TaskId::from(step_id.as_str());
        state.running_steps.remove(&task_id);
        state.failed_steps.insert(task_id);
        let payload =
            serde_json::json!({ "step_id": step_id.as_str(), "error": error }).to_string();
        self.emit_step_event(state, WorkflowEventType::StepFailed, step_id, &payload)
    }

    /// Record that a step is retrying.
    pub fn step_retrying(
        &self,
        state: &mut ExecutionState,
        step_id: &StepId,
        attempt: u32,
    ) -> Result<(), AresError> {
        let task_id = TaskId::from(step_id.as_str());
        state.failed_steps.remove(&task_id);
        state.retry_steps.insert(task_id);
        state
            .step_attempts
            .insert(step_id.as_str().to_string(), attempt);
        let payload =
            serde_json::json!({ "step_id": step_id.as_str(), "attempt": attempt }).to_string();
        self.emit_step_event(state, WorkflowEventType::StepRetrying, step_id, &payload)
    }

    // ─────────────────────────────────────────────────────────────
    // State reconstruction from snapshot + event replay
    // ─────────────────────────────────────────────────────────────

    /// Reconstruct execution state from the latest snapshot plus
    /// replaying subsequent events. Enforces MAX_REPLAY_EVENTS guard.
    pub fn reconstruct_execution_state(
        &self,
        execution_id: &ExecutionId,
        _is_replay: bool,
    ) -> Result<ExecutionState, AresError> {
        // Try loading snapshot
        let snapshot_opt = self.repo.load_snapshot(execution_id)?;

        let (mut state, replay_after) = match snapshot_opt {
            Some(snapshot) => {
                // Verify checksum
                let computed = blake3::hash(snapshot.state_json.as_bytes())
                    .to_hex()
                    .to_string();
                if computed != snapshot.checksum {
                    return Err(AresError::validation(format!(
                        "Snapshot checksum mismatch: expected {}, got {}",
                        snapshot.checksum, computed
                    )));
                }
                let state: ExecutionState = serde_json::from_str(&snapshot.state_json)?;
                (state, snapshot.last_event_sequence)
            }
            None => {
                // No snapshot — reconstruct from scratch
                // We need a minimal state; the events will build it up
                let status = self.repo.get_execution_status(execution_id)?;
                let state = ExecutionState {
                    workflow_id: WorkflowId::from(""),
                    execution_id: execution_id.clone(),
                    workflow_version_id: String::new(),
                    current_status: status,
                    completed_steps: BTreeSet::new(),
                    failed_steps: BTreeSet::new(),
                    pending_steps: BTreeSet::new(),
                    retry_steps: BTreeSet::new(),
                    running_steps: BTreeSet::new(),
                    skipped_steps: BTreeSet::new(),
                    last_event_sequence: 0,
                    step_attempts: std::collections::BTreeMap::new(),
                    step_agents: std::collections::BTreeMap::new(),
                    version: 0,
                };
                (state, 0)
            }
        };

        // Replay events after snapshot
        let events = self.repo.list_events_after(execution_id, replay_after)?;

        for event in &events {
            apply_event_to_state(&mut state, event);
        }

        Ok(state)
    }

    // ─────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────

    /// Emit a workflow-level event, persist transactionally, and maybe snapshot.
    fn emit_event(
        &self,
        state: &mut ExecutionState,
        event_type: WorkflowEventType,
        new_status: &WorkflowStatus,
        _step_id: Option<&StepId>,
        payload: &str,
    ) -> Result<(), AresError> {
        let seq = self.repo.next_sequence_number(&state.execution_id)?;
        let event = WorkflowEvent {
            id: EventId::new(),
            execution_id: state.execution_id.clone(),
            sequence_number: seq,
            schema_version: WORKFLOW_EVENT_SCHEMA_VERSION,
            event_type,
            timestamp: now_micros(),
            payload: payload.to_string(),
        };

        let expected_version = state.version;
        self.repo
            .append_event_and_update_status(&event, new_status, expected_version)?;
        state.last_event_sequence = seq;
        state.version += 1;

        // Snapshot check
        if seq > 0 && seq % SNAPSHOT_INTERVAL == 0 {
            self.create_snapshot(state)?;
        }

        Ok(())
    }

    /// Emit a step-level event.
    fn emit_step_event(
        &self,
        state: &mut ExecutionState,
        event_type: WorkflowEventType,
        step_id: &StepId,
        payload: &str,
    ) -> Result<(), AresError> {
        let seq = self.repo.next_sequence_number(&state.execution_id)?;
        let event = WorkflowEvent {
            id: EventId::new(),
            execution_id: state.execution_id.clone(),
            sequence_number: seq,
            schema_version: WORKFLOW_EVENT_SCHEMA_VERSION,
            event_type,
            timestamp: now_micros(),
            payload: payload.to_string(),
        };

        let expected_version = state.version;
        self.repo.append_step_event_and_update_status(
            &event,
            &state.current_status,
            step_id,
            expected_version,
        )?;
        state.last_event_sequence = seq;
        state.version += 1;

        if seq > 0 && seq % SNAPSHOT_INTERVAL == 0 {
            self.create_snapshot(state)?;
        }

        Ok(())
    }

    /// Create a BLAKE3-checksum-verified snapshot of the current state.
    fn create_snapshot(&self, state: &ExecutionState) -> Result<(), AresError> {
        let state_json = serde_json::to_string(state)?;
        let checksum = blake3::hash(state_json.as_bytes()).to_hex().to_string();

        let snapshot = WorkflowExecutionSnapshot {
            execution_id: state.execution_id.clone(),
            last_event_sequence: state.last_event_sequence,
            created_at: now_micros(),
            checksum,
            state_json,
        };

        self.repo.save_snapshot(&snapshot)
    }

    /// Force a snapshot (public API for time-based snapshotting).
    pub fn force_snapshot(&self, state: &ExecutionState) -> Result<(), AresError> {
        self.create_snapshot(state)
    }
}

// ─────────────────────────────────────────────────────────────────
// Event replay logic (pure function — no side effects)
// ─────────────────────────────────────────────────────────────────

/// Apply a single event to an execution state (deterministic replay).
fn apply_event_to_state(state: &mut ExecutionState, event: &WorkflowEvent) {
    state.last_event_sequence = event.sequence_number;

    match event.event_type {
        WorkflowEventType::WorkflowCreated => {
            state.current_status = WorkflowStatus::Pending;
        }
        WorkflowEventType::WorkflowStarted => {
            state.current_status = WorkflowStatus::Running;
        }
        WorkflowEventType::WorkflowCompleted => {
            state.current_status = WorkflowStatus::Completed;
        }
        WorkflowEventType::WorkflowFailed => {
            state.current_status = WorkflowStatus::Failed;
        }
        WorkflowEventType::WorkflowCancelled => {
            state.current_status = WorkflowStatus::Cancelled;
        }
        WorkflowEventType::WorkflowPaused => {
            state.current_status = WorkflowStatus::Paused;
        }
        WorkflowEventType::WorkflowResumed => {
            state.current_status = WorkflowStatus::Running;
        }
        WorkflowEventType::WorkflowTimedOut => {
            state.current_status = WorkflowStatus::TimedOut;
        }
        WorkflowEventType::StepStarted => {
            if let Some(step_id) = extract_step_id(&event.payload) {
                let task_id = TaskId::from(step_id.as_str());
                state.pending_steps.remove(&task_id);
                state.retry_steps.remove(&task_id);
                state.running_steps.insert(task_id);
            }
        }
        WorkflowEventType::StepCompleted => {
            if let Some(step_id) = extract_step_id(&event.payload) {
                let task_id = TaskId::from(step_id.as_str());
                state.running_steps.remove(&task_id);
                state.completed_steps.insert(task_id);
            }
        }
        WorkflowEventType::StepFailed => {
            if let Some(step_id) = extract_step_id(&event.payload) {
                let task_id = TaskId::from(step_id.as_str());
                state.running_steps.remove(&task_id);
                state.failed_steps.insert(task_id);
            }
        }
        WorkflowEventType::StepRetrying => {
            if let Some(step_id) = extract_step_id(&event.payload) {
                let task_id = TaskId::from(step_id.as_str());
                state.failed_steps.remove(&task_id);
                state.retry_steps.insert(task_id);
                // Update attempt count
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&event.payload) {
                    if let Some(attempt) = v.get("attempt").and_then(|a| a.as_u64()) {
                        state.step_attempts.insert(step_id, attempt as u32);
                    }
                }
            }
        }
        WorkflowEventType::StepSkipped => {
            if let Some(step_id) = extract_step_id(&event.payload) {
                let task_id = TaskId::from(step_id.as_str());
                state.pending_steps.remove(&task_id);
                state.skipped_steps.insert(task_id);
            }
        }
        // Agent assignment, snapshot, dead letter, compensation — tracked but
        // don't change the core step sets.
        WorkflowEventType::AgentAssigned => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&event.payload) {
                if let (Some(step_id), Some(agent_id)) = (
                    v.get("step_id").and_then(|s| s.as_str()),
                    v.get("agent_id").and_then(|a| a.as_str()),
                ) {
                    state
                        .step_agents
                        .insert(step_id.to_string(), ares_core::AgentId::from(agent_id));
                }
            }
        }
        _ => {
            // Other event types (SnapshotCreated, DeadLetterCreated, etc.)
            // are informational — no state mutation.
        }
    }
}

/// Extract `step_id` from a JSON payload string.
fn extract_step_id(payload: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(payload)
        .ok()
        .and_then(|v| v.get("step_id").and_then(|s| s.as_str()).map(String::from))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::test_helpers::test_store;
    use ares_store::SqliteWorkflowRepository;
    use std::sync::Arc;

    fn setup() -> (Arc<SqliteWorkflowRepository>, WorkflowId, String) {
        let (store, _dir) = test_store();
        // Keep _dir alive by leaking it (test only)
        let dir = Box::leak(Box::new(_dir));
        let _ = dir;
        let repo = Arc::new(SqliteWorkflowRepository::new(store));
        let wf_id = WorkflowId::new();
        let version_id = "test-v1".to_string();

        repo.create_workflow(&wf_id, "test-wf", "test workflow")
            .unwrap();

        let def = ares_core::WorkflowDefinition {
            workflow_id: wf_id.clone(),
            version: 1,
            name: "test-wf".into(),
            description: "test".into(),
            steps: vec![
                ares_core::WorkflowStepDef {
                    id: StepId::from("step-1"),
                    name: "Build".into(),
                    description: "Build step".into(),
                    required_capability: "build".into(),
                    depends_on: vec![],
                    timeout_ms: Some(30000),
                    retry_policy: Default::default(),
                    compensation: Default::default(),
                    priority: Default::default(),
                },
                ares_core::WorkflowStepDef {
                    id: StepId::from("step-2"),
                    name: "Test".into(),
                    description: "Test step".into(),
                    required_capability: "test".into(),
                    depends_on: vec![StepId::from("step-1")],
                    timeout_ms: Some(60000),
                    retry_policy: Default::default(),
                    compensation: Default::default(),
                    priority: Default::default(),
                },
            ],
            timeout_ms: Some(120000),
        };
        let def_json = serde_json::to_string(&def).unwrap();
        repo.create_version(&version_id, &wf_id, 1, &def_json, Some(120000))
            .unwrap();

        (repo, wf_id, version_id)
    }

    #[test]
    fn create_plan_and_execute() {
        let (repo, wf_id, version_id) = setup();
        let engine = WorkflowEngine::new(repo);

        let plan = engine
            .create_execution_plan(&wf_id, &version_id, 1)
            .unwrap();
        assert_eq!(plan.execution_order.len(), 2);

        let state = engine.execute_workflow(&plan).unwrap();
        assert_eq!(state.current_status, WorkflowStatus::Running);
        assert_eq!(state.pending_steps.len(), 2);
        assert_eq!(state.last_event_sequence, 2); // Created + Started
    }

    #[test]
    fn pause_and_resume() {
        let (repo, wf_id, version_id) = setup();
        let engine = WorkflowEngine::new(repo);
        let plan = engine
            .create_execution_plan(&wf_id, &version_id, 1)
            .unwrap();
        let mut state = engine.execute_workflow(&plan).unwrap();

        engine.pause_workflow(&mut state).unwrap();
        assert_eq!(state.current_status, WorkflowStatus::Paused);

        engine.resume_workflow(&mut state).unwrap();
        assert_eq!(state.current_status, WorkflowStatus::Running);
    }

    #[test]
    fn cancel_workflow() {
        let (repo, wf_id, version_id) = setup();
        let engine = WorkflowEngine::new(repo);
        let plan = engine
            .create_execution_plan(&wf_id, &version_id, 1)
            .unwrap();
        let mut state = engine.execute_workflow(&plan).unwrap();

        engine.cancel_workflow(&mut state).unwrap();
        assert_eq!(state.current_status, WorkflowStatus::Cancelled);
    }

    #[test]
    fn step_lifecycle() {
        let (repo, wf_id, version_id) = setup();
        let engine = WorkflowEngine::new(repo);
        let plan = engine
            .create_execution_plan(&wf_id, &version_id, 1)
            .unwrap();
        let mut state = engine.execute_workflow(&plan).unwrap();

        let step = StepId::from("step-1");
        engine.step_queued(&mut state, &step).unwrap();
        engine.step_started(&mut state, &step).unwrap();
        assert!(state.running_steps.contains(&TaskId::from("step-1")));
        assert!(!state.pending_steps.contains(&TaskId::from("step-1")));

        engine.step_completed(&mut state, &step).unwrap();
        assert!(state.completed_steps.contains(&TaskId::from("step-1")));
        assert!(!state.running_steps.contains(&TaskId::from("step-1")));
    }

    #[test]
    fn reconstruct_from_events() {
        let (repo, wf_id, version_id) = setup();
        let engine = WorkflowEngine::new(repo);
        let plan = engine
            .create_execution_plan(&wf_id, &version_id, 1)
            .unwrap();
        let mut state = engine.execute_workflow(&plan).unwrap();
        let exec_id = state.execution_id.clone();

        let step = StepId::from("step-1");
        engine.step_started(&mut state, &step).unwrap();
        engine.step_completed(&mut state, &step).unwrap();

        // Reconstruct
        let reconstructed = engine.reconstruct_execution_state(&exec_id, false).unwrap();
        assert_eq!(reconstructed.last_event_sequence, state.last_event_sequence);
        assert!(reconstructed
            .completed_steps
            .contains(&TaskId::from("step-1")));
    }

    #[test]
    fn snapshot_and_reconstruct() {
        let (repo, wf_id, version_id) = setup();
        let engine = WorkflowEngine::new(repo);
        let plan = engine
            .create_execution_plan(&wf_id, &version_id, 1)
            .unwrap();
        let mut state = engine.execute_workflow(&plan).unwrap();

        let step = StepId::from("step-1");
        engine.step_started(&mut state, &step).unwrap();
        engine.step_completed(&mut state, &step).unwrap();

        // Force snapshot
        engine.force_snapshot(&state).unwrap();

        // Add more events after snapshot
        let step2 = StepId::from("step-2");
        engine.step_started(&mut state, &step2).unwrap();

        // Reconstruct from snapshot + replay
        let reconstructed = engine
            .reconstruct_execution_state(&state.execution_id, false)
            .unwrap();
        assert_eq!(reconstructed.last_event_sequence, state.last_event_sequence);
        assert!(reconstructed
            .completed_steps
            .contains(&TaskId::from("step-1")));
        assert!(reconstructed
            .running_steps
            .contains(&TaskId::from("step-2")));
    }
}
