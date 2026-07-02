use crate::planner::builder::ExecutionPlan;
use crate::planner::dag::{ExecutionGraph, ExecutionNode};

pub struct DependencyResolver;

impl DependencyResolver {
    #[tracing::instrument(name = "DependencyResolver::resolve", skip(plan))]
    pub fn resolve(plan: &ExecutionPlan) -> ExecutionGraph {
        let start = std::time::Instant::now();
        // Simple mock for now
        let mut graph = ExecutionGraph::new();
        for cap in &plan.requested_capabilities {
            graph.nodes.push(ExecutionNode {
                capability: cap.clone(),
                dependencies: vec![],
            });
        }
        tracing::debug!(
            duration_ms = start.elapsed().as_millis(),
            nodes = graph.nodes.len(),
            "Resolved dependencies"
        );
        graph
    }
}
