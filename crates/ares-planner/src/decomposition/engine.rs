use crate::models::goal::Goal;
use ares_core::AresError;

pub struct GoalDecompositionEngine;

impl GoalDecompositionEngine {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Decomposes a top-level goal into a sequence of actionable subgoals.
    /// In the future, this will use an LLM or predefined templates.
    pub fn decompose(&self, goal: &Goal) -> Result<Vec<Goal>, AresError> {
        // Placeholder: For now, return a single sub-goal mimicking the main goal.
        // Week 13 Multi-Model intelligence will actually hook up the reasoning engine here.

        let sub_goal = Goal {
            id: ares_core::id::GoalId::new(),
            title: format!("Step 1 of {}", goal.title),
            description: goal.description.clone(),
            priority: goal.priority.clone(),
            deadline: goal.deadline,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Ok(vec![sub_goal])
    }

    pub fn decompose_to_candidates(
        &self,
        _goal: &Goal,
    ) -> Result<Vec<crate::models::candidate::PlanCandidate>, AresError> {
        let candidate = crate::models::candidate::PlanCandidate {
            id: ares_core::id::PlanCandidateId::new(),
            goal_id: _goal.id.clone(),
            dag_json: r#"{"nodes":[],"edges":[]}"#.to_string(),
            score: 0.0,
            estimated_cost: None,
            estimated_duration: None,
            generated_at: chrono::Utc::now(),
        };
        Ok(vec![candidate])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::goal::GoalPriority;
    use ares_core::id::GoalId;
    use chrono::Utc;

    fn make_test_goal() -> Goal {
        Goal {
            id: GoalId::new(),
            title: "Test Goal".to_string(),
            description: Some("Test".to_string()),
            priority: GoalPriority::High,
            deadline: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_decompose_returns_subgoals() {
        let engine = GoalDecompositionEngine::new();
        let goal = make_test_goal();
        let subgoals = engine.decompose(&goal).unwrap();

        assert_eq!(subgoals.len(), 1);
        assert_eq!(subgoals[0].title, "Step 1 of Test Goal");
    }

    #[test]
    fn test_decompose_to_candidates() {
        let engine = GoalDecompositionEngine::new();
        let goal = make_test_goal();
        let candidates = engine.decompose_to_candidates(&goal).unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].goal_id, goal.id);
        assert_eq!(candidates[0].dag_json, r#"{"nodes":[],"edges":[]}"#);
    }
}
