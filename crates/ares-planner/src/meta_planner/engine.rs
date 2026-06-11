use crate::meta_planner::models::{
    ComplexityEstimate, MissionType, PlanningIntent, PlanningStrategy,
};
use crate::models::goal::Goal;
use ares_core::AresError;
use chrono::Utc;

/// The Meta Planner analyses high-level goals and produces a `PlanningIntent`
/// that drives downstream decomposition and strategy selection.
pub struct MetaPlanner;

impl Default for MetaPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaPlanner {
    pub fn new() -> Self {
        Self
    }

    /// Full analysis pipeline: classify → estimate → select → emit intent.
    pub fn analyze_goal(&self, goal: &Goal) -> Result<PlanningIntent, AresError> {
        let mission_type = self.classify_mission_type(goal);
        let complexity = self.estimate_complexity(goal);
        let strategy = self.select_strategy(&mission_type, &complexity);
        let estimated_steps = self.estimate_step_count(&complexity);

        Ok(PlanningIntent {
            goal_id: goal.id.clone(),
            mission_type,
            complexity,
            strategy,
            constraints: Vec::new(),
            estimated_steps,
            created_at: Utc::now(),
        })
    }

    /// Classify the mission type from the goal's title and description
    /// using keyword matching.
    pub fn classify_mission_type(&self, goal: &Goal) -> MissionType {
        let text = Self::goal_text(goal);

        // Order matters: more specific patterns first.
        let patterns: &[(&[&str], MissionType)] = &[
            (
                &[
                    "debug",
                    "fix bug",
                    "troubleshoot",
                    "diagnose",
                    "stack trace",
                ],
                MissionType::Debugging,
            ),
            (
                &[
                    "refactor",
                    "restructure",
                    "reorganize",
                    "clean up",
                    "modernize",
                ],
                MissionType::Refactoring,
            ),
            (
                &[
                    "deploy",
                    "release",
                    "ship",
                    "ci/cd",
                    "pipeline",
                    "infrastructure",
                ],
                MissionType::Deployment,
            ),
            (
                &["research", "investigate", "explore", "survey", "literature"],
                MissionType::Research,
            ),
            (
                &[
                    "analyze",
                    "analysis",
                    "evaluate",
                    "assess",
                    "benchmark",
                    "profile",
                ],
                MissionType::Analysis,
            ),
            (
                &[
                    "build",
                    "implement",
                    "create",
                    "develop",
                    "code",
                    "write",
                    "program",
                    "function",
                    "module",
                    "api",
                    "endpoint",
                ],
                MissionType::Coding,
            ),
        ];

        for (keywords, mission_type) in patterns {
            for kw in *keywords {
                if text.contains(kw) {
                    return mission_type.clone();
                }
            }
        }

        // Fallback: if the description is long or contains multiple verbs,
        // treat as a multi-step project.
        if goal.description.as_deref().unwrap_or("").len() > 200 {
            return MissionType::MultiStepProject;
        }

        MissionType::Coding // safe default
    }

    /// Estimate complexity based on description length, keyword density,
    /// and the presence of multi-step indicators.
    pub fn estimate_complexity(&self, goal: &Goal) -> ComplexityEstimate {
        let desc = goal.description.as_deref().unwrap_or("");
        let desc_len = desc.len();
        let word_count = desc.split_whitespace().count();

        // Multi-step indicators
        let step_indicators = [
            "then",
            "after",
            "next",
            "finally",
            "first",
            "second",
            "third",
            "phase",
            "stage",
            "step",
            "milestone",
        ];
        let step_count = step_indicators
            .iter()
            .filter(|kw| desc.to_lowercase().contains(**kw))
            .count();

        if desc_len < 30 && step_count == 0 {
            ComplexityEstimate::Trivial
        } else if word_count < 20 && step_count <= 1 {
            ComplexityEstimate::Simple
        } else if word_count < 60 && step_count <= 3 {
            ComplexityEstimate::Moderate
        } else if word_count < 150 || step_count <= 5 {
            ComplexityEstimate::Complex
        } else {
            ComplexityEstimate::Epic
        }
    }

    /// Select a planning strategy based on mission type and complexity.
    pub fn select_strategy(
        &self,
        mission_type: &MissionType,
        complexity: &ComplexityEstimate,
    ) -> PlanningStrategy {
        match (mission_type, complexity) {
            // Simple tasks → sequential
            (_, ComplexityEstimate::Trivial) => PlanningStrategy::Sequential,
            (_, ComplexityEstimate::Simple) => PlanningStrategy::Sequential,

            // Debugging benefits from iteration
            (MissionType::Debugging, _) => PlanningStrategy::Iterative,

            // Multi-step projects need hierarchy
            (MissionType::MultiStepProject, ComplexityEstimate::Epic) => PlanningStrategy::Adaptive,
            (MissionType::MultiStepProject, _) => PlanningStrategy::Hierarchical,

            // Complex coding / deployment can parallelise
            (MissionType::Coding, ComplexityEstimate::Complex | ComplexityEstimate::Epic) => {
                PlanningStrategy::Parallel
            }
            (MissionType::Deployment, ComplexityEstimate::Complex | ComplexityEstimate::Epic) => {
                PlanningStrategy::Parallel
            }

            // Everything else moderate+ → hierarchical
            (_, ComplexityEstimate::Moderate) => PlanningStrategy::Hierarchical,
            (_, _) => PlanningStrategy::Hierarchical,
        }
    }

    // ── private helpers ────────────────────────────────────────────

    fn goal_text(goal: &Goal) -> String {
        let mut text = goal.title.to_lowercase();
        if let Some(ref desc) = goal.description {
            text.push(' ');
            text.push_str(&desc.to_lowercase());
        }
        text
    }

    fn estimate_step_count(&self, complexity: &ComplexityEstimate) -> u32 {
        match complexity {
            ComplexityEstimate::Trivial => 1,
            ComplexityEstimate::Simple => 3,
            ComplexityEstimate::Moderate => 5,
            ComplexityEstimate::Complex => 8,
            ComplexityEstimate::Epic => 13,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::goal::{Goal, GoalPriority};
    use ares_core::id::GoalId;

    fn make_goal(title: &str, desc: Option<&str>) -> Goal {
        Goal {
            id: GoalId::new(),
            title: title.to_string(),
            description: desc.map(|s| s.to_string()),
            priority: GoalPriority::High,
            deadline: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ── classify_mission_type ────────────────────────────────────

    #[test]
    fn classify_research_goal() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Research new database options", None);
        assert_eq!(mp.classify_mission_type(&goal), MissionType::Research);
    }

    #[test]
    fn classify_coding_goal() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Implement user authentication", None);
        assert_eq!(mp.classify_mission_type(&goal), MissionType::Coding);
    }

    #[test]
    fn classify_refactoring_goal() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Refactor the payment module", None);
        assert_eq!(mp.classify_mission_type(&goal), MissionType::Refactoring);
    }

    #[test]
    fn classify_debugging_goal() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Debug the login failure", None);
        assert_eq!(mp.classify_mission_type(&goal), MissionType::Debugging);
    }

    #[test]
    fn classify_deployment_goal() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Deploy to production", None);
        assert_eq!(mp.classify_mission_type(&goal), MissionType::Deployment);
    }

    #[test]
    fn classify_analysis_goal() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Analyze performance bottlenecks", None);
        assert_eq!(mp.classify_mission_type(&goal), MissionType::Analysis);
    }

    #[test]
    fn classify_multi_step_from_long_description() {
        let mp = MetaPlanner::new();
        let long_desc = "a ".repeat(150); // 300 chars
        let goal = make_goal("Do something", Some(&long_desc));
        assert_eq!(
            mp.classify_mission_type(&goal),
            MissionType::MultiStepProject
        );
    }

    #[test]
    fn classify_defaults_to_coding() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Something vague", Some("short"));
        assert_eq!(mp.classify_mission_type(&goal), MissionType::Coding);
    }

    #[test]
    fn classify_uses_description_keywords() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Task", Some("investigate the root cause of the crash"));
        assert_eq!(mp.classify_mission_type(&goal), MissionType::Research);
    }

    // ── estimate_complexity ──────────────────────────────────────

    #[test]
    fn trivial_complexity_short_desc() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Fix typo", Some("small fix"));
        assert_eq!(mp.estimate_complexity(&goal), ComplexityEstimate::Trivial);
    }

    #[test]
    fn simple_complexity() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Add logging", Some("Add structured logging to the api"));
        assert_eq!(mp.estimate_complexity(&goal), ComplexityEstimate::Simple);
    }

    #[test]
    fn moderate_complexity() {
        let mp = MetaPlanner::new();
        let goal = make_goal(
            "Refactor auth",
            Some("First refactor the token service then update the middleware and after that write tests for the new flow"),
        );
        assert_eq!(mp.estimate_complexity(&goal), ComplexityEstimate::Moderate);
    }

    #[test]
    fn complex_or_epic_complexity() {
        let mp = MetaPlanner::new();
        let words = "word ".repeat(160);
        let goal = make_goal("Big project", Some(&words));
        let complexity = mp.estimate_complexity(&goal);
        assert!(complexity >= ComplexityEstimate::Complex);
    }

    #[test]
    fn no_description_is_trivial() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Quick task", None);
        assert_eq!(mp.estimate_complexity(&goal), ComplexityEstimate::Trivial);
    }

    // ── select_strategy ──────────────────────────────────────────

    #[test]
    fn trivial_gets_sequential() {
        let mp = MetaPlanner::new();
        assert_eq!(
            mp.select_strategy(&MissionType::Coding, &ComplexityEstimate::Trivial),
            PlanningStrategy::Sequential
        );
    }

    #[test]
    fn debug_gets_iterative() {
        let mp = MetaPlanner::new();
        assert_eq!(
            mp.select_strategy(&MissionType::Debugging, &ComplexityEstimate::Complex),
            PlanningStrategy::Iterative
        );
    }

    #[test]
    fn epic_multiproject_gets_adaptive() {
        let mp = MetaPlanner::new();
        assert_eq!(
            mp.select_strategy(&MissionType::MultiStepProject, &ComplexityEstimate::Epic),
            PlanningStrategy::Adaptive
        );
    }

    #[test]
    fn complex_coding_gets_parallel() {
        let mp = MetaPlanner::new();
        assert_eq!(
            mp.select_strategy(&MissionType::Coding, &ComplexityEstimate::Complex),
            PlanningStrategy::Parallel
        );
    }

    // ── analyze_goal (integration) ───────────────────────────────

    #[test]
    fn analyze_goal_produces_intent() {
        let mp = MetaPlanner::new();
        let goal = make_goal("Build a REST API", Some("Implement CRUD endpoints"));
        let intent = mp.analyze_goal(&goal).unwrap();

        assert_eq!(intent.goal_id, goal.id);
        assert_eq!(intent.mission_type, MissionType::Coding);
        assert!(intent.estimated_steps >= 1);
    }

    #[test]
    fn analyze_goal_complex_project() {
        let mp = MetaPlanner::new();
        let goal = make_goal(
            "Build a production React dashboard",
            Some(
                "First research the best framework then build the frontend \
                 after that implement the backend and finally deploy to production \
                 with CI/CD pipeline. Phase one covers auth, phase two covers data.",
            ),
        );
        let intent = mp.analyze_goal(&goal).unwrap();
        assert!(intent.complexity >= ComplexityEstimate::Moderate);
        assert!(intent.estimated_steps >= 5);
    }

    #[test]
    fn analyze_preserves_goal_id() {
        let mp = MetaPlanner::new();
        let goal = make_goal("test", None);
        let intent = mp.analyze_goal(&goal).unwrap();
        assert_eq!(intent.goal_id, goal.id);
    }

    #[test]
    fn step_count_matches_complexity() {
        let mp = MetaPlanner::new();
        assert_eq!(mp.estimate_step_count(&ComplexityEstimate::Trivial), 1);
        assert_eq!(mp.estimate_step_count(&ComplexityEstimate::Epic), 13);
    }

    #[test]
    fn default_trait() {
        let mp = MetaPlanner::new();
        let goal = make_goal("test", None);
        assert!(mp.analyze_goal(&goal).is_ok());
    }
}
