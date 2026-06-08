use ares_core::types::workflow_api::{ExecutionSearchRequest, ExecutionSummary};
use ares_core::{
    AgentInfo, AresError, DeadLetterEntry, ExecutionId, WorkflowEvent, WorkflowExecutionSnapshot,
    WorkflowStatus,
};

pub trait WorkflowRepository: Send + Sync {
    // Agent Operations
    fn register_agent(
        &self,
        id: &str,
        name: &str,
        capabilities_json: &str,
        health_json: &str,
        performance_json: &str,
    ) -> Result<(), AresError>;
    fn list_agents(&self) -> Result<Vec<AgentInfo>, AresError>;
    fn update_agent_health(&self, id: &str, health_json: &str) -> Result<(), AresError>;
    fn update_agent_performance(&self, id: &str, performance_json: &str) -> Result<(), AresError>;

    // Workflow & Execution Operations
    fn create_workflow(
        &self,
        id: &ares_core::WorkflowId,
        name: &str,
        description: &str,
    ) -> Result<(), AresError>;
    fn create_version(
        &self,
        version_id: &str,
        workflow_id: &ares_core::WorkflowId,
        version: u32,
        definition_json: &str,
        timeout_ms: Option<u64>,
    ) -> Result<(), AresError>;
    fn get_version_definition(&self, version_id: &str) -> Result<String, AresError>;

    // Transactional boundaries for executions
    fn start_workflow_execution(
        &self,
        execution_id: &ExecutionId,
        workflow_version_id: &str,
        events: Vec<WorkflowEvent>,
        status: &WorkflowStatus,
    ) -> Result<(), AresError>;
    fn create_execution(
        &self,
        execution_id: &ExecutionId,
        workflow_version_id: &str,
    ) -> Result<(), AresError>;
    fn append_event_and_update_status(
        &self,
        event: &WorkflowEvent,
        new_status: &WorkflowStatus,
        expected_version: u64,
    ) -> Result<(), AresError>;
    fn append_step_event_and_update_status(
        &self,
        event: &WorkflowEvent,
        new_status: &WorkflowStatus,
        step_id: &ares_core::StepId,
        expected_version: u64,
    ) -> Result<(), AresError>;
    fn complete_execution(
        &self,
        execution_id: &ExecutionId,
        new_status: &WorkflowStatus,
    ) -> Result<(), AresError>;
    fn get_execution_status(&self, execution_id: &ExecutionId)
        -> Result<WorkflowStatus, AresError>;
    fn next_sequence_number(&self, execution_id: &ExecutionId) -> Result<u64, AresError>;
    fn list_events_after(
        &self,
        execution_id: &ExecutionId,
        seq: u64,
    ) -> Result<Vec<WorkflowEvent>, AresError>;
    fn count_events(&self, execution_id: &ExecutionId) -> Result<u64, AresError>;

    // Snapshots & Dead Letters
    fn save_snapshot(&self, snapshot: &WorkflowExecutionSnapshot) -> Result<(), AresError>;
    fn load_snapshot(
        &self,
        execution_id: &ExecutionId,
    ) -> Result<Option<WorkflowExecutionSnapshot>, AresError>;
    fn insert_dead_letter(&self, entry: &DeadLetterEntry) -> Result<(), AresError>;
    fn list_dead_letters(&self, limit: u32) -> Result<Vec<DeadLetterEntry>, AresError>;

    // Analytics & Visualization
    fn update_analytics_cache(&self, duration_ms: f64, success: bool) -> Result<(), AresError>;
    fn get_analytics_cache(&self) -> Result<(u64, u64, u64), AresError>;
    fn get_visualization(&self, version_id: &str) -> Result<Option<String>, AresError>;
    fn save_visualization(&self, version_id: &str, graph_json: &str) -> Result<(), AresError>;

    // Phase 4.1 Search Executions
    fn search_executions(
        &self,
        req: &ExecutionSearchRequest,
    ) -> Result<(Vec<ExecutionSummary>, u64), AresError>;
}
