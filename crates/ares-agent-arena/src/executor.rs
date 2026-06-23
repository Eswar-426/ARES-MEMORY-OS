use crate::agents::baseline::BaselineAgent;
use crate::agents::context_aware::ContextAwareAgent;
use crate::agents::enhanced::EnhancedContextAgent;
use crate::agents::planner::PlannerAgentStub;
use crate::agents::AgentRunner;
use crate::evaluator::AgentEvaluator;
use crate::models::ArenaTask;
use crate::report::ArenaReport;
use anyhow::Result;

pub struct ArenaExecutor {
    pub baseline: BaselineAgent,
    pub context_aware: ContextAwareAgent,
    pub enhanced: EnhancedContextAgent,
    pub planner: PlannerAgentStub,
}

impl ArenaExecutor {
    pub async fn execute_task(&self, task: &ArenaTask) -> Result<ArenaReport> {
        let mut report = ArenaReport {
            task_id: task.id.clone(),
            baseline: None,
            context: None,
            enhanced: None,
            planner: None,
        };

        // Run Baseline
        let base_res = self.baseline.run(task).await?;
        report.baseline = Some(AgentEvaluator::evaluate(task, base_res));

        // Run ContextAware
        let ctx_res = self.context_aware.run(task).await?;
        report.context = Some(AgentEvaluator::evaluate(task, ctx_res));

        // Run Enhanced
        let enh_res = self.enhanced.run(task).await?;
        report.enhanced = Some(AgentEvaluator::evaluate(task, enh_res));

        // Run Planner
        let plan_res = self.planner.run(task).await?;
        report.planner = Some(AgentEvaluator::evaluate(task, plan_res));

        Ok(report)
    }
}
