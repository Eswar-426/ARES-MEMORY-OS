use crate::approval::service::ApprovalService;
use crate::decomposition::engine::GoalDecompositionEngine;
use crate::evaluation::scoring::PlanScoringService;
use crate::explain::service::ExplainabilityService;
use crate::guardrails::validator::GuardrailValidator;
use crate::model_selection::service::ModelSelectionService;
use crate::models::approval::ApprovalMode;
use crate::models::candidate::PlanCandidate;
use crate::models::goal::Goal;
use crate::simulation::service::SimulationService;
use crate::workflow_mapper::translator::WorkflowTranslator;
use ares_core::id::PlanId;
use ares_core::types::workflow::WorkflowDefinition;
use ares_core::AresError;
use std::sync::Arc;

pub struct PlannerCoordinator {
    decomposition: Arc<GoalDecompositionEngine>,
    guardrails: Arc<GuardrailValidator>,
    model_selection: Arc<ModelSelectionService>,
    simulation: Arc<SimulationService>,
    scoring: Arc<PlanScoringService>,
    approval: Arc<ApprovalService>,
    explain: Arc<ExplainabilityService>,
    workflow: Arc<WorkflowTranslator>,
}

impl PlannerCoordinator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        decomposition: Arc<GoalDecompositionEngine>,
        guardrails: Arc<GuardrailValidator>,
        model_selection: Arc<ModelSelectionService>,
        simulation: Arc<SimulationService>,
        scoring: Arc<PlanScoringService>,
        approval: Arc<ApprovalService>,
        explain: Arc<ExplainabilityService>,
        workflow: Arc<WorkflowTranslator>,
    ) -> Self {
        Self {
            decomposition,
            guardrails,
            model_selection,
            simulation,
            scoring,
            approval,
            explain,
            workflow,
        }
    }

    /// The master function to generate a plan for a goal.
    pub fn generate_plan(&self, goal: &Goal) -> Result<PlanId, AresError> {
        // 1. Validate Goal Constraints
        self.guardrails.validate_goal(goal)?;

        // 2. Select LLMs for the generation task
        let _model = self.model_selection.select_best_model("general")?;

        // 3. Decompose Goal -> PlanCandidates (Simulated for now)
        let candidates = self.decomposition.decompose_to_candidates(goal)?;

        // 4. Simulate & Score Candidates
        let mut scored_candidates = Vec::new();
        for mut candidate in candidates {
            let estimates = self.simulation.simulate(&candidate)?;
            candidate.estimated_cost = Some(estimates.expected_cost);
            candidate.estimated_duration = Some(estimates.expected_duration_seconds);

            let score = self.scoring.score_plan(
                &estimates,
                &crate::evaluation::scoring::OptimizationObjective::LowestCost,
            );
            candidate.score = score;
            scored_candidates.push(candidate);
        }

        // Sort by score descending
        scored_candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best_candidate = scored_candidates
            .first()
            .ok_or_else(|| AresError::Validation("No valid plan candidates generated".into()))?;

        // 5. Generate Explanation
        let _explanation = self.explain.explain_selection(
            &ares_core::id::PlanId::new(),
            best_candidate,
            &scored_candidates,
            "cost",
        )?;

        // 6. Approval Process
        let approval = self.approval.process_approval(
            None,
            best_candidate,
            ApprovalMode::Hybrid, // From configuration
        )?;

        Ok(approval.plan_id)
    }

    /// Maps an approved plan to a workflow definition and passes to execution.
    pub fn map_and_execute_plan(
        &self,
        _plan_id: &PlanId,
        candidate: &PlanCandidate,
    ) -> Result<WorkflowDefinition, AresError> {
        // In reality we fetch the Plan and PlanDag from repository.
        // We'll mock deserializing the dag_json here:
        let dag: crate::dag::models::PlanDag = serde_json::from_str(&candidate.dag_json)
            .map_err(|e| AresError::Serialization(e.to_string()))?;

        let workflow = self.workflow.translate(
            &dag,
            format!("Workflow for Candidate {}", candidate.id),
            "Auto-generated".into(),
        )?;

        // The caller (or API) will send this to ares-orchestrator.
        Ok(workflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::goal::GoalPriority;
    use ares_core::id::GoalId;
    use chrono::Utc;

    #[test]
    fn test_coordinator_generates_workflow() {
        let db_path = std::env::temp_dir().join(format!("test_db_{}.sqlite", uuid::Uuid::new_v4()));
        let store = ares_store::db::Store::open(&db_path).unwrap();
        let store_arc = Arc::new(store.clone());

        let repo = Arc::new(crate::repository::approvals::SqliteApprovalRepository::new(
            store_arc,
        ));
        let event_repo = Arc::new(ares_store::repositories::event::SqliteEventRepository::new(
            store,
        ));
        let publisher = Arc::new(crate::events::publisher::PlannerEventPublisher::new(
            event_repo,
        ));

        let coordinator = PlannerCoordinator::new(
            Arc::new(GoalDecompositionEngine::new()),
            Arc::new(GuardrailValidator),
            Arc::new(ModelSelectionService::new()),
            Arc::new(SimulationService::new()),
            Arc::new(PlanScoringService::new()),
            Arc::new(ApprovalService::new(repo, publisher)),
            Arc::new(ExplainabilityService::new()),
            Arc::new(WorkflowTranslator::new()),
        );

        let goal = Goal {
            id: GoalId::new(),
            title: "Integration Goal".to_string(),
            description: Some("Test".to_string()),
            priority: GoalPriority::Medium,
            deadline: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Complete lifecycle from Goal -> PlanId
        let plan_id = coordinator.generate_plan(&goal).unwrap();

        let candidate =
            PlanCandidate::new_test(plan_id.clone(), r#"{"nodes":[],"edges":[]}"#.to_string());

        // Mapping to workflow
        let workflow = coordinator
            .map_and_execute_plan(&plan_id, &candidate)
            .unwrap();

        assert_eq!(workflow.steps.len(), 0);
        assert!(workflow.name.contains(&candidate.id.to_string()));
    }
}
