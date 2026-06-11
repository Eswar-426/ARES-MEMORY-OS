use crate::decomposition::goal_dag::{GoalDag, GoalNode};
use crate::decomposition::recursive::GoalDecomposer;
use crate::meta_planner::engine::MetaPlanner;
use crate::meta_planner::models::PlanningIntent;
use crate::models::goal::{Goal, GoalPriority};
use crate::strategy::engine::StrategyEngine;
use crate::strategy::models::{HistoricalPerformance, StrategyConstraints, StrategySelection};
use ares_core::id::GoalId;
use ares_core::AresError;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// The complete output of mission generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedMission {
    pub goal: Goal,
    pub intent: PlanningIntent,
    pub goal_dag: GoalDag,
    pub strategy: StrategySelection,
}

/// Generates executable missions from natural language goals by
/// composing the Meta Planner, Goal Decomposer, and Strategy Engine.
pub struct MissionGenerator {
    meta_planner: MetaPlanner,
    decomposer: GoalDecomposer,
    strategy_engine: StrategyEngine,
}

impl Default for MissionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl MissionGenerator {
    pub fn new() -> Self {
        Self {
            meta_planner: MetaPlanner::new(),
            decomposer: GoalDecomposer::new(),
            strategy_engine: StrategyEngine::new(),
        }
    }

    /// Full pipeline: parse goal → meta plan → decompose → select strategy.
    pub fn generate_mission(&self, goal_text: &str) -> Result<GeneratedMission, AresError> {
        self.generate_mission_with_history(goal_text, &StrategyConstraints::default(), &[])
    }

    /// Full pipeline with constraints and historical performance data.
    pub fn generate_mission_with_history(
        &self,
        goal_text: &str,
        constraints: &StrategyConstraints,
        history: &[HistoricalPerformance],
    ) -> Result<GeneratedMission, AresError> {
        // 1. Create a Goal from the text
        let goal = self.parse_goal(goal_text);

        // 2. Analyze with Meta Planner
        let intent = self.meta_planner.analyze_goal(&goal)?;

        // 3. Decompose into Goal DAG
        let goal_dag = self.decomposer.decompose_recursive(&goal, &intent)?;

        // 4. Select execution strategy
        let strategy =
            self.strategy_engine
                .select_strategy(constraints, &intent.complexity, history);

        Ok(GeneratedMission {
            goal,
            intent,
            goal_dag,
            strategy,
        })
    }

    /// Assign an agent role based on the goal node's title keywords.
    pub fn assign_role(&self, node: &GoalNode) -> String {
        let title_lower = node.title.to_lowercase();

        let role_keywords: &[(&[&str], &str)] = &[
            (&["architect", "design", "plan", "structure"], "Architect"),
            (
                &["research", "investigate", "survey", "literature"],
                "Researcher",
            ),
            (&["test", "verify", "validate", "check", "qa"], "Tester"),
            (&["security", "vulnerability", "threat"], "Security"),
            (&["review", "audit", "inspect"], "Reviewer"),
            (&["document", "doc", "readme", "report"], "Documentation"),
            (&["deploy", "release", "ship", "ci/cd"], "DevOps"),
        ];

        for (keywords, role) in role_keywords {
            for kw in *keywords {
                if title_lower.contains(kw) {
                    return role.to_string();
                }
            }
        }

        "Coder".to_string() // default
    }

    // ── Private helpers ──────────────────────────────────────────

    fn parse_goal(&self, text: &str) -> Goal {
        // Split on first sentence boundary or newline for title vs description
        let (title, description) = if let Some(pos) = text.find(['.', '\n']) {
            let t = text[..pos].trim().to_string();
            let d = text[pos + 1..].trim().to_string();
            (t, if d.is_empty() { None } else { Some(d) })
        } else {
            (text.trim().to_string(), None)
        };

        Goal {
            id: GoalId::new(),
            title,
            description,
            priority: GoalPriority::High,
            deadline: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_simple_mission() {
        let gen = MissionGenerator::new();
        let mission = gen.generate_mission("Build a REST API").unwrap();

        assert_eq!(mission.goal.title, "Build a REST API");
        assert!(mission.goal_dag.node_count() >= 2);
        assert!(mission.goal_dag.validate_acyclic());
    }

    #[test]
    fn generate_complex_mission() {
        let gen = MissionGenerator::new();
        let mission = gen
            .generate_mission(
                "Build a production React dashboard. \
                 First set up the project, then implement the frontend components, \
                 after that build the backend API, then write tests, \
                 and finally deploy to production.",
            )
            .unwrap();

        assert!(mission.goal_dag.node_count() >= 3);
        assert!(mission.goal_dag.validate_acyclic());
        assert!(mission.intent.estimated_steps >= 3);
    }

    #[test]
    fn generate_research_mission() {
        let gen = MissionGenerator::new();
        let mission = gen
            .generate_mission("Research the best database for our use case")
            .unwrap();

        assert_eq!(
            mission.intent.mission_type,
            crate::meta_planner::models::MissionType::Research
        );
    }

    #[test]
    fn generate_debug_mission() {
        let gen = MissionGenerator::new();
        let mission = gen
            .generate_mission("Debug the login crash on production")
            .unwrap();

        assert_eq!(
            mission.intent.mission_type,
            crate::meta_planner::models::MissionType::Debugging
        );
    }

    #[test]
    fn parse_goal_with_description() {
        let gen = MissionGenerator::new();
        let goal = gen.parse_goal("Build auth. Implement JWT-based authentication");
        assert_eq!(goal.title, "Build auth");
        assert_eq!(
            goal.description.as_deref(),
            Some("Implement JWT-based authentication")
        );
    }

    #[test]
    fn parse_goal_no_description() {
        let gen = MissionGenerator::new();
        let goal = gen.parse_goal("Fix the bug");
        assert_eq!(goal.title, "Fix the bug");
        assert!(goal.description.is_none());
    }

    #[test]
    fn assign_role_architect() {
        let gen = MissionGenerator::new();
        let node = GoalNode {
            id: GoalId::new(),
            title: "Design architecture for auth".to_string(),
            description: None,
            dependencies: vec![],
            priority: GoalPriority::High,
            estimated_cost: 0.0,
            estimated_duration_secs: 0.0,
            depth: 0,
        };
        assert_eq!(gen.assign_role(&node), "Architect");
    }

    #[test]
    fn assign_role_tester() {
        let gen = MissionGenerator::new();
        let node = GoalNode {
            id: GoalId::new(),
            title: "Write tests for API".to_string(),
            description: None,
            dependencies: vec![],
            priority: GoalPriority::Medium,
            estimated_cost: 0.0,
            estimated_duration_secs: 0.0,
            depth: 0,
        };
        assert_eq!(gen.assign_role(&node), "Tester");
    }

    #[test]
    fn assign_role_default_coder() {
        let gen = MissionGenerator::new();
        let node = GoalNode {
            id: GoalId::new(),
            title: "Implement the feature".to_string(),
            description: None,
            dependencies: vec![],
            priority: GoalPriority::High,
            estimated_cost: 0.0,
            estimated_duration_secs: 0.0,
            depth: 0,
        };
        assert_eq!(gen.assign_role(&node), "Coder");
    }

    #[test]
    fn assign_role_researcher() {
        let gen = MissionGenerator::new();
        let node = GoalNode {
            id: GoalId::new(),
            title: "Research database options".to_string(),
            description: None,
            dependencies: vec![],
            priority: GoalPriority::High,
            estimated_cost: 0.0,
            estimated_duration_secs: 0.0,
            depth: 0,
        };
        assert_eq!(gen.assign_role(&node), "Researcher");
    }

    #[test]
    fn assign_role_security() {
        let gen = MissionGenerator::new();
        let node = GoalNode {
            id: GoalId::new(),
            title: "Security audit".to_string(),
            description: None,
            dependencies: vec![],
            priority: GoalPriority::Critical,
            estimated_cost: 0.0,
            estimated_duration_secs: 0.0,
            depth: 0,
        };
        assert_eq!(gen.assign_role(&node), "Security");
    }

    #[test]
    fn assign_role_documentation() {
        let gen = MissionGenerator::new();
        let node = GoalNode {
            id: GoalId::new(),
            title: "Write documentation".to_string(),
            description: None,
            dependencies: vec![],
            priority: GoalPriority::Low,
            estimated_cost: 0.0,
            estimated_duration_secs: 0.0,
            depth: 0,
        };
        assert_eq!(gen.assign_role(&node), "Documentation");
    }

    #[test]
    fn generate_with_constraints() {
        let gen = MissionGenerator::new();
        let constraints = StrategyConstraints {
            budget: Some(50.0),
            deadline_secs: Some(3600.0),
            ..Default::default()
        };
        let mission = gen
            .generate_mission_with_history("Build an API", &constraints, &[])
            .unwrap();

        // Should have picked up the constraints
        assert!(mission.strategy.constraints_applied.budget.is_some());
    }

    #[test]
    fn generated_mission_serialization() {
        let gen = MissionGenerator::new();
        let mission = gen.generate_mission("Build auth").unwrap();
        let json = serde_json::to_string(&mission).unwrap();
        let back: GeneratedMission = serde_json::from_str(&json).unwrap();
        assert_eq!(back.goal.title, mission.goal.title);
    }

    #[test]
    fn default_trait() {
        let gen = MissionGenerator::default();
        assert!(gen.generate_mission("test").is_ok());
    }

    #[test]
    fn goal_dag_is_always_acyclic() {
        let gen = MissionGenerator::new();
        let inputs = [
            "Build API",
            "Debug crash",
            "Research options",
            "Deploy to prod",
            "Refactor the codebase",
        ];
        for input in inputs {
            let mission = gen.generate_mission(input).unwrap();
            assert!(
                mission.goal_dag.validate_acyclic(),
                "DAG for '{}' has a cycle!",
                input
            );
        }
    }
}
