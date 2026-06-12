use ares_agent_runtime::models::{AgentId, TaskId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::assignment::{DelegationId, DelegationRecord, DelegationStatus};
use super::splitter::{SplitResult, TaskSplitter};
use crate::governor::SafetyGovernor;

/// Engine for delegating tasks between agents with governor checks.
pub struct DelegationEngine {
    records: Arc<RwLock<HashMap<DelegationId, DelegationRecord>>>,
    splitter: TaskSplitter,
}

impl DelegationEngine {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            splitter: TaskSplitter::new(),
        }
    }

    /// Assign a task from one agent to another.
    pub async fn assign_task(
        &self,
        task_id: TaskId,
        from: AgentId,
        to: AgentId,
        depth: u32,
        reason: impl Into<String>,
        governor: Option<&SafetyGovernor>,
    ) -> Result<DelegationId, String> {
        // Governor check
        if let Some(gov) = governor {
            let decision = gov.check_delegation(depth).await;
            if decision.is_denied() {
                return Err(format!("Governor denied delegation: {:?}", decision));
            }
        }

        let record = DelegationRecord::new(task_id, from, to, depth, reason);
        let id = record.id;
        self.records.write().await.insert(id, record);
        Ok(id)
    }

    /// Reassign a delegated task to a different agent.
    pub async fn reassign_task(
        &self,
        delegation_id: &DelegationId,
        new_agent: AgentId,
    ) -> Result<DelegationId, String> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(delegation_id) {
            let old_record = record.clone();
            record.status = DelegationStatus::Reassigned;
            record.completed_at = Some(chrono::Utc::now().timestamp());

            // Create new delegation
            let new_record = DelegationRecord::new(
                old_record.task_id,
                old_record.from_agent,
                new_agent,
                old_record.depth,
                format!("Reassigned from {:?}", old_record.to_agent.0),
            );
            let new_id = new_record.id;
            records.insert(new_id, new_record);
            Ok(new_id)
        } else {
            Err(format!("Delegation {:?} not found", delegation_id))
        }
    }

    /// Split a task into sub-tasks for parallel or sequential delegation.
    pub fn split_task(&self, description: &str, complexity: f64) -> SplitResult {
        self.splitter.analyze(description, complexity)
    }

    /// Merge results from multiple sub-task delegations.
    pub async fn merge_results(&self, delegation_ids: &[DelegationId]) -> Result<String, String> {
        let records = self.records.read().await;
        let mut results = Vec::new();

        for id in delegation_ids {
            if let Some(record) = records.get(id) {
                if record.status != DelegationStatus::Completed {
                    return Err(format!("Delegation {:?} not completed", id));
                }
                if let Some(result) = &record.result {
                    results.push(result.clone());
                }
            } else {
                return Err(format!("Delegation {:?} not found", id));
            }
        }

        Ok(results.join("\n---\n"))
    }

    /// Escalate a delegation to the assigning agent's parent.
    pub async fn escalate_task(
        &self,
        delegation_id: &DelegationId,
        reason: impl Into<String>,
    ) -> Result<(), String> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(delegation_id) {
            record.escalate();
            record.result = Some(reason.into());
            Ok(())
        } else {
            Err(format!("Delegation {:?} not found", delegation_id))
        }
    }

    /// Complete a delegation with a result.
    pub async fn complete_delegation(
        &self,
        delegation_id: &DelegationId,
        result: impl Into<String>,
    ) -> Result<(), String> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(delegation_id) {
            record.complete(result);
            Ok(())
        } else {
            Err(format!("Delegation {:?} not found", delegation_id))
        }
    }

    /// Fail a delegation with a reason.
    pub async fn fail_delegation(
        &self,
        delegation_id: &DelegationId,
        reason: impl Into<String>,
    ) -> Result<(), String> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(delegation_id) {
            record.fail(reason);
            Ok(())
        } else {
            Err(format!("Delegation {:?} not found", delegation_id))
        }
    }

    /// Get a delegation record.
    pub async fn get_delegation(&self, id: &DelegationId) -> Option<DelegationRecord> {
        self.records.read().await.get(id).cloned()
    }

    /// Get all delegations for a specific task.
    pub async fn get_delegations_for_task(&self, task_id: &TaskId) -> Vec<DelegationRecord> {
        self.records
            .read()
            .await
            .values()
            .filter(|r| r.task_id == *task_id)
            .cloned()
            .collect()
    }

    /// Get delegation count.
    pub async fn delegation_count(&self) -> usize {
        self.records.read().await.len()
    }
}

impl Default for DelegationEngine {
    fn default() -> Self {
        Self::new()
    }
}
