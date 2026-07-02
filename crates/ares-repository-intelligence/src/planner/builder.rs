use crate::core::capabilities::Capability;
use crate::planner::intent::Intent;

pub struct ExecutionPlan {
    pub plan_id: String,
    pub intent: Intent,
    pub requested_capabilities: Vec<Capability>,
    pub streaming: bool,
    pub priority: i32,
}

pub struct PlanBuilder;

impl PlanBuilder {
    #[tracing::instrument(name = "PlanBuilder::build", skip(intent), fields(intent = ?intent))]
    pub fn build(intent: Intent) -> ExecutionPlan {
        let start = std::time::Instant::now();
        let mut caps = Vec::new();

        match intent {
            Intent::ExplainEntity => {
                caps.push(Capability::WhyExists);
            }
            Intent::AnalyzeImpact => {
                caps.push(Capability::ImpactAnalysis);
            }
            Intent::FindPath => {
                caps.push(Capability::GraphSearch);
            }
            Intent::Dashboard => {
                caps.push(Capability::Workspace);
            }
            Intent::Traceability => {
                caps.push(Capability::Traceability);
            }
            Intent::GeneralQuestion => {
                caps.push(Capability::GraphSearch);
                caps.push(Capability::Knowledge);
            }
            Intent::Unknown => {}
        }

        let plan = ExecutionPlan {
            plan_id: uuid::Uuid::new_v4().to_string(),
            intent,
            requested_capabilities: caps,
            streaming: false,
            priority: 0,
        };
        tracing::debug!(duration_ms = start.elapsed().as_millis(), plan_id = %plan.plan_id, "Plan built");
        plan
    }
}
