use crate::explain::engine::{ExplainabilityEngine, PlanExplanation};
use crate::models::candidate::PlanCandidate;
use ares_core::id::PlanId;
use ares_core::AresError;

pub struct ExplainabilityService {
    engine: ExplainabilityEngine,
}

impl ExplainabilityService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            engine: ExplainabilityEngine::new(),
        }
    }

    pub fn explain_selection(
        &self,
        plan_id: &PlanId,
        selected: &PlanCandidate,
        all_candidates: &[PlanCandidate],
        objective: &str,
    ) -> Result<PlanExplanation, AresError> {
        let explanation =
            self.engine
                .generate_explanation(plan_id, selected, all_candidates, objective);

        // TODO: Save PlanExplanation to a dedicated explainability repository/store.

        Ok(explanation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_selection() {
        let service = ExplainabilityService::new();
        let plan_id = PlanId::new();

        let mut selected = PlanCandidate::new_test(plan_id.clone(), "{}".to_string());
        selected.score = 0.95;
        selected.estimated_cost = Some(10.5);
        selected.estimated_duration = Some(120.0);

        let alt1 = PlanCandidate::new_test(plan_id.clone(), "{}".to_string());
        let alt2 = PlanCandidate::new_test(plan_id.clone(), "{}".to_string());

        let all_candidates = vec![selected.clone(), alt1, alt2];

        let explanation = service
            .explain_selection(&plan_id, &selected, &all_candidates, "LowestCost")
            .unwrap();

        assert_eq!(explanation.plan_id, plan_id);
        assert_eq!(explanation.alternative_candidates_considered, 2);
        assert_eq!(explanation.primary_objective, "LowestCost");
        assert!(explanation.reasoning.contains("LowestCost"));
        assert!(explanation.reasoning.contains("10.5"));
        assert!(explanation.reasoning.contains("120"));
        assert!(explanation.reasoning.contains("0.95"));
    }
}
