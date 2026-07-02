use crate::core::context::RepositoryContext;
use crate::core::engine::{EngineExecutionResult, EngineInput};
use crate::core::errors::EngineResult;
use crate::planner::dag::ExecutionGraph;
use crate::planner::executor::EngineExecutor;

pub struct Scheduler;

impl Scheduler {
    #[tracing::instrument(name = "Scheduler::execute_graph", skip(graph, executor, context))]
    pub async fn execute_graph(
        graph: &ExecutionGraph,
        executor: &EngineExecutor<'_>,
        context: &RepositoryContext,
    ) -> EngineResult<Vec<EngineExecutionResult>> {
        let start = std::time::Instant::now();

        let mut futures = Vec::new();
        for node in &graph.nodes {
            // For now, execute all nodes independently in parallel (mock DAG traversal)
            let fut = executor.execute(&node.capability, context, EngineInput::None);
            futures.push(fut);
        }

        // Execute parallel tasks
        let raw_results = futures::future::join_all(futures).await;

        let mut results = Vec::new();
        for r in raw_results {
            results.extend(r?); // Flatten the Vec<EngineExecutionResult> from each capability
        }

        tracing::debug!(
            duration_ms = start.elapsed().as_millis(),
            tasks_executed = results.len(),
            "Execution completed"
        );
        Ok(results)
    }
}
