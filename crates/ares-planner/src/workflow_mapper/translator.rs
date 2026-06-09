use crate::dag::models::PlanDag;
use ares_core::id::{StepId, WorkflowId};
use ares_core::types::workflow::{
    CompensationAction, RetryPolicy, TaskPriority, WorkflowDefinition, WorkflowStepDef,
};
use ares_core::AresError;
use std::collections::HashMap;

pub struct WorkflowTranslator;

impl WorkflowTranslator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Maps a Planner `PlanDag` to an Orchestrator `WorkflowDefinition`.
    /// This enforces the philosophy: "Planner plans. Orchestrator executes."
    pub fn translate(
        &self,
        dag: &PlanDag,
        name: String,
        description: String,
    ) -> Result<WorkflowDefinition, AresError> {
        let mut steps = Vec::new();
        let mut node_to_step_id: HashMap<String, StepId> = HashMap::new();

        // Pass 1: Create Step IDs
        for node in &dag.nodes {
            node_to_step_id.insert(node.id.clone(), StepId::new());
        }

        // Pass 2: Map edges to dependencies
        let mut dependencies: HashMap<String, Vec<StepId>> = HashMap::new();
        for edge in &dag.edges {
            if let Some(dep_step_id) = node_to_step_id.get(&edge.source) {
                dependencies
                    .entry(edge.target.clone())
                    .or_default()
                    .push(dep_step_id.clone());
            }
        }

        // Pass 3: Construct StepDefs
        for node in &dag.nodes {
            let step_id = node_to_step_id[&node.id].clone();
            let depends_on = dependencies.get(&node.id).cloned().unwrap_or_default();

            let step = WorkflowStepDef {
                id: step_id,
                name: node.title.clone(),
                description: format!("Generated from DAG node {}", node.id),
                required_capability: "general".to_string(), // In reality, pulled from knowledge
                depends_on,
                timeout_ms: Some((node.estimated_duration * 1000.0) as u64),
                retry_policy: RetryPolicy::default(),
                compensation: CompensationAction::None,
                priority: TaskPriority::Normal,
            };

            steps.push(step);
        }

        let workflow = WorkflowDefinition {
            workflow_id: WorkflowId::new(),
            version: 1,
            name,
            description,
            steps,
            timeout_ms: None,
        };

        Ok(workflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::models::{DagEdge, DagNode};

    #[test]
    fn test_translate_dag_to_workflow() {
        let translator = WorkflowTranslator::new();
        let dag = PlanDag {
            nodes: vec![
                DagNode {
                    id: "A".to_string(),
                    title: "Task A".to_string(),
                    estimated_duration: 10.0,
                    cost: 0.0,
                },
                DagNode {
                    id: "B".to_string(),
                    title: "Task B".to_string(),
                    estimated_duration: 20.0,
                    cost: 0.0,
                },
            ],
            edges: vec![DagEdge {
                source: "A".to_string(),
                target: "B".to_string(),
            }],
        };

        let workflow = translator
            .translate(&dag, "Test WF".to_string(), "Desc".to_string())
            .unwrap();

        assert_eq!(workflow.name, "Test WF");
        assert_eq!(workflow.steps.len(), 2);

        // Find Step B
        let step_b = workflow.steps.iter().find(|s| s.name == "Task B").unwrap();
        assert_eq!(step_b.depends_on.len(), 1); // Depends on Step A

        // Timeout should be duration * 1000
        assert_eq!(step_b.timeout_ms, Some(20000));
    }
}
