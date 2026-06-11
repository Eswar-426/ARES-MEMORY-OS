use crate::decomposition::goal_dag::{GoalDag, GoalEdge, GoalEdgeType, GoalNode};
use crate::meta_planner::models::{ComplexityEstimate, MissionType, PlanningIntent};
use crate::models::goal::{Goal, GoalPriority};
use ares_core::id::GoalId;
use ares_core::AresError;

/// Recursively decomposes a high-level goal into a Goal DAG using
/// the `PlanningIntent` from the Meta Planner to decide depth and shape.
pub struct GoalDecomposer;

impl Default for GoalDecomposer {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalDecomposer {
    pub fn new() -> Self {
        Self
    }

    /// Decompose a goal into a DAG driven by the planning intent.
    pub fn decompose_recursive(
        &self,
        goal: &Goal,
        intent: &PlanningIntent,
    ) -> Result<GoalDag, AresError> {
        let max_depth = intent.complexity.max_decomposition_depth();
        let mut dag = GoalDag::new(goal.id.clone());

        // Create root node from the goal
        let root = GoalNode {
            id: goal.id.clone(),
            title: goal.title.clone(),
            description: goal.description.clone(),
            dependencies: Vec::new(),
            priority: goal.priority.clone(),
            estimated_cost: self.base_cost(&intent.complexity),
            estimated_duration_secs: self.base_duration(&intent.complexity),
            depth: 0,
        };
        dag.add_node(root);

        // Generate sub-goals based on mission type
        let subtasks = self.generate_subtasks(&intent.mission_type, &goal.title);

        if subtasks.is_empty() {
            return Ok(dag);
        }

        // Create sub-goal nodes and chain them
        let mut prev_id: Option<GoalId> = None;
        for (i, (title, desc)) in subtasks.iter().enumerate() {
            let depth = 1;
            if depth > max_depth {
                break;
            }

            let node_id = GoalId::new();
            let node = GoalNode {
                id: node_id.clone(),
                title: title.clone(),
                description: Some(desc.clone()),
                dependencies: prev_id.iter().cloned().collect(),
                priority: self.priority_for_index(i, subtasks.len()),
                estimated_cost: self.base_cost(&intent.complexity) / subtasks.len() as f64,
                estimated_duration_secs: self.base_duration(&intent.complexity)
                    / subtasks.len() as f64,
                depth,
            };

            dag.add_node(node);

            // Edge from root to child
            dag.add_edge(GoalEdge {
                from: goal.id.clone(),
                to: node_id.clone(),
                edge_type: GoalEdgeType::SubGoal,
            });

            // Sequential dependency edge (if not first)
            if let Some(ref prev) = prev_id {
                dag.add_edge(GoalEdge {
                    from: prev.clone(),
                    to: node_id.clone(),
                    edge_type: GoalEdgeType::DependsOn,
                });
            }

            prev_id = Some(node_id);
        }

        // Generate milestones if complex enough
        if intent.complexity >= ComplexityEstimate::Moderate {
            let milestones = self.generate_milestones(&dag);
            for ms in milestones {
                let ms_id = ms.id.clone();
                // Find the dependency node
                let deps = ms.dependencies.clone();
                dag.add_node(ms);
                for dep in deps {
                    dag.add_edge(GoalEdge {
                        from: dep,
                        to: ms_id.clone(),
                        edge_type: GoalEdgeType::Milestone,
                    });
                }
            }
        }

        Ok(dag)
    }

    /// Extract dependencies between nodes based on keyword ordering heuristics.
    pub fn extract_dependencies(&self, nodes: &[GoalNode]) -> Vec<GoalEdge> {
        let mut edges = Vec::new();
        // Simple heuristic: nodes that mention keywords from earlier nodes depend on them
        for i in 1..nodes.len() {
            let prev_title_lower = nodes[i - 1].title.to_lowercase();
            let curr_desc = nodes[i].description.as_deref().unwrap_or("").to_lowercase();

            // If current description mentions previous task's title keywords, add dependency
            let prev_words: Vec<&str> = prev_title_lower
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();

            let has_reference = prev_words.iter().any(|w| curr_desc.contains(w));
            if has_reference {
                edges.push(GoalEdge {
                    from: nodes[i - 1].id.clone(),
                    to: nodes[i].id.clone(),
                    edge_type: GoalEdgeType::DependsOn,
                });
            }
        }
        edges
    }

    /// Insert milestone checkpoint nodes at mid-points of the DAG.
    pub fn generate_milestones(&self, dag: &GoalDag) -> Vec<GoalNode> {
        let leaves = dag.get_leaves();
        if leaves.is_empty() {
            return Vec::new();
        }

        // Create a milestone after the last leaf
        let last_leaf = &leaves[leaves.len() - 1];
        let milestone = GoalNode {
            id: GoalId::new(),
            title: format!("Milestone: {} complete", dag.root_id),
            description: Some("Checkpoint — all preceding tasks finished".to_string()),
            dependencies: vec![last_leaf.clone()],
            priority: GoalPriority::High,
            estimated_cost: 0.0,
            estimated_duration_secs: 0.0,
            depth: 0,
        };

        vec![milestone]
    }

    // ── Private helpers ──────────────────────────────────────────

    fn generate_subtasks(
        &self,
        mission_type: &MissionType,
        goal_title: &str,
    ) -> Vec<(String, String)> {
        match mission_type {
            MissionType::Coding => vec![
                (
                    format!("Design architecture for {}", goal_title),
                    "Define component structure, interfaces, and data flow".to_string(),
                ),
                (
                    format!("Implement core logic for {}", goal_title),
                    "Write the main implementation code".to_string(),
                ),
                (
                    format!("Write tests for {}", goal_title),
                    "Create comprehensive unit and integration tests".to_string(),
                ),
                (
                    format!("Code review for {}", goal_title),
                    "Review implementation for quality and correctness".to_string(),
                ),
            ],
            MissionType::Research => vec![
                (
                    format!("Literature survey for {}", goal_title),
                    "Gather existing knowledge and prior art".to_string(),
                ),
                (
                    format!("Analysis of findings for {}", goal_title),
                    "Synthesize and evaluate gathered information".to_string(),
                ),
                (
                    format!("Report for {}", goal_title),
                    "Produce final research report with recommendations".to_string(),
                ),
            ],
            MissionType::Debugging => vec![
                (
                    format!("Reproduce issue: {}", goal_title),
                    "Create reliable reproduction steps".to_string(),
                ),
                (
                    format!("Root cause analysis: {}", goal_title),
                    "Identify the underlying cause of the bug".to_string(),
                ),
                (
                    format!("Fix: {}", goal_title),
                    "Implement the fix for the identified root cause".to_string(),
                ),
                (
                    format!("Verify fix: {}", goal_title),
                    "Confirm the fix resolves the issue without regressions".to_string(),
                ),
            ],
            MissionType::Deployment => vec![
                (
                    format!("Pre-deployment checks for {}", goal_title),
                    "Verify all prerequisites are met".to_string(),
                ),
                (
                    format!("Deploy {}", goal_title),
                    "Execute the deployment procedure".to_string(),
                ),
                (
                    format!("Post-deployment verification for {}", goal_title),
                    "Validate deployment success and health".to_string(),
                ),
            ],
            MissionType::Refactoring => vec![
                (
                    format!("Identify refactoring targets in {}", goal_title),
                    "Map code that needs restructuring".to_string(),
                ),
                (
                    format!("Refactor {}", goal_title),
                    "Apply structural improvements".to_string(),
                ),
                (
                    format!("Validate refactoring of {}", goal_title),
                    "Ensure behaviour is preserved after refactoring".to_string(),
                ),
            ],
            MissionType::Analysis => vec![
                (
                    format!("Data collection for {}", goal_title),
                    "Gather relevant data and metrics".to_string(),
                ),
                (
                    format!("Analysis of {}", goal_title),
                    "Perform the core analysis".to_string(),
                ),
                (
                    format!("Conclusions for {}", goal_title),
                    "Summarise findings and recommendations".to_string(),
                ),
            ],
            MissionType::MultiStepProject => vec![
                (
                    format!("Planning for {}", goal_title),
                    "Create detailed project plan".to_string(),
                ),
                (
                    format!("Phase 1 of {}", goal_title),
                    "Execute the first major phase".to_string(),
                ),
                (
                    format!("Phase 2 of {}", goal_title),
                    "Execute the second major phase".to_string(),
                ),
                (
                    format!("Integration for {}", goal_title),
                    "Integrate all components".to_string(),
                ),
                (
                    format!("Final validation of {}", goal_title),
                    "End-to-end validation and sign-off".to_string(),
                ),
            ],
        }
    }

    fn priority_for_index(&self, index: usize, total: usize) -> GoalPriority {
        if index == 0 {
            GoalPriority::Critical
        } else if index < total / 2 {
            GoalPriority::High
        } else {
            GoalPriority::Medium
        }
    }

    fn base_cost(&self, complexity: &ComplexityEstimate) -> f64 {
        complexity.weight() * 20.0
    }

    fn base_duration(&self, complexity: &ComplexityEstimate) -> f64 {
        complexity.weight() * 300.0 // seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta_planner::models::PlanningStrategy;
    use chrono::Utc;

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

    fn make_intent(goal: &Goal, mt: MissionType, cx: ComplexityEstimate) -> PlanningIntent {
        PlanningIntent {
            goal_id: goal.id.clone(),
            mission_type: mt,
            complexity: cx,
            strategy: PlanningStrategy::Sequential,
            constraints: Vec::new(),
            estimated_steps: 5,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn decompose_coding_goal() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Build auth module", Some("Implement JWT auth"));
        let intent = make_intent(&goal, MissionType::Coding, ComplexityEstimate::Moderate);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        // Root + 4 coding subtasks + milestone(s)
        assert!(dag.node_count() >= 5);
        assert!(dag.validate_acyclic());
    }

    #[test]
    fn decompose_research_goal() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Research DB options", None);
        let intent = make_intent(&goal, MissionType::Research, ComplexityEstimate::Simple);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        assert!(dag.node_count() >= 4); // root + 3 research subtasks
        assert!(dag.validate_acyclic());
    }

    #[test]
    fn decompose_debugging_goal() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Fix login crash", None);
        let intent = make_intent(&goal, MissionType::Debugging, ComplexityEstimate::Moderate);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        assert!(dag.node_count() >= 5); // root + 4 debug tasks + milestone
        assert!(dag.validate_acyclic());
    }

    #[test]
    fn decompose_deployment_goal() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Deploy v2.0", None);
        let intent = make_intent(&goal, MissionType::Deployment, ComplexityEstimate::Simple);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        assert!(dag.node_count() >= 4); // root + 3 deploy tasks
    }

    #[test]
    fn decompose_multi_step_project() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Build full dashboard", None);
        let intent = make_intent(
            &goal,
            MissionType::MultiStepProject,
            ComplexityEstimate::Complex,
        );

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        assert!(dag.node_count() >= 6); // root + 5 phases + milestones
        assert!(dag.validate_acyclic());
    }

    #[test]
    fn root_node_has_goal_id() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Test", None);
        let intent = make_intent(&goal, MissionType::Coding, ComplexityEstimate::Trivial);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        assert!(dag.nodes.contains_key(&goal.id));
    }

    #[test]
    fn trivial_complexity_limits_depth() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Quick fix", None);
        let intent = make_intent(&goal, MissionType::Coding, ComplexityEstimate::Trivial);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        for node in dag.nodes.values() {
            assert!(node.depth <= 1);
        }
    }

    #[test]
    fn moderate_complexity_generates_milestones() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Medium project", None);
        let intent = make_intent(&goal, MissionType::Coding, ComplexityEstimate::Moderate);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        let has_milestone = dag
            .edges
            .iter()
            .any(|e| e.edge_type == GoalEdgeType::Milestone);
        assert!(has_milestone);
    }

    #[test]
    fn simple_complexity_no_milestones() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Simple task", None);
        let intent = make_intent(&goal, MissionType::Coding, ComplexityEstimate::Simple);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        let has_milestone = dag
            .edges
            .iter()
            .any(|e| e.edge_type == GoalEdgeType::Milestone);
        assert!(!has_milestone);
    }

    #[test]
    fn sequential_dependencies_between_subtasks() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Build API", None);
        let intent = make_intent(&goal, MissionType::Coding, ComplexityEstimate::Moderate);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        let depends_edges = dag
            .edges
            .iter()
            .filter(|e| e.edge_type == GoalEdgeType::DependsOn)
            .count();
        // Should have sequential deps between subtasks
        assert!(depends_edges >= 2);
    }

    #[test]
    fn extract_dependencies_keyword_match() {
        let decomposer = GoalDecomposer::new();
        let n1 = GoalNode {
            id: GoalId::new(),
            title: "Build authentication".to_string(),
            description: Some("Create auth module".to_string()),
            dependencies: vec![],
            priority: GoalPriority::High,
            estimated_cost: 10.0,
            estimated_duration_secs: 60.0,
            depth: 0,
        };
        let n2 = GoalNode {
            id: GoalId::new(),
            title: "Test endpoints".to_string(),
            description: Some("Test the authentication endpoints".to_string()),
            dependencies: vec![],
            priority: GoalPriority::Medium,
            estimated_cost: 5.0,
            estimated_duration_secs: 30.0,
            depth: 0,
        };

        let edges = decomposer.extract_dependencies(&[n1, n2]);
        assert!(!edges.is_empty());
    }

    #[test]
    fn extract_dependencies_no_match() {
        let decomposer = GoalDecomposer::new();
        let n1 = GoalNode {
            id: GoalId::new(),
            title: "A".to_string(),
            description: Some("xyz".to_string()),
            dependencies: vec![],
            priority: GoalPriority::Low,
            estimated_cost: 1.0,
            estimated_duration_secs: 10.0,
            depth: 0,
        };
        let n2 = GoalNode {
            id: GoalId::new(),
            title: "B".to_string(),
            description: Some("abc".to_string()),
            dependencies: vec![],
            priority: GoalPriority::Low,
            estimated_cost: 1.0,
            estimated_duration_secs: 10.0,
            depth: 0,
        };

        let edges = decomposer.extract_dependencies(&[n1, n2]);
        assert!(edges.is_empty());
    }

    #[test]
    fn default_trait() {
        let d = GoalDecomposer::new();
        let goal = make_goal("test", None);
        let intent = make_intent(&goal, MissionType::Coding, ComplexityEstimate::Trivial);
        assert!(d.decompose_recursive(&goal, &intent).is_ok());
    }

    #[test]
    fn first_subtask_has_critical_priority() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Build API", None);
        let intent = make_intent(&goal, MissionType::Coding, ComplexityEstimate::Simple);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        // Find non-root nodes
        let children: Vec<_> = dag.nodes.values().filter(|n| n.id != goal.id).collect();
        assert!(!children.is_empty());
        // At least one should be Critical
        assert!(children
            .iter()
            .any(|n| n.priority == GoalPriority::Critical));
    }

    #[test]
    fn refactoring_goal_subtasks() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Refactor payments", None);
        let intent = make_intent(&goal, MissionType::Refactoring, ComplexityEstimate::Simple);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        assert!(dag.node_count() >= 4); // root + 3 refactoring subtasks
    }

    #[test]
    fn analysis_goal_subtasks() {
        let decomposer = GoalDecomposer::new();
        let goal = make_goal("Analyze latency", None);
        let intent = make_intent(&goal, MissionType::Analysis, ComplexityEstimate::Simple);

        let dag = decomposer.decompose_recursive(&goal, &intent).unwrap();
        assert!(dag.node_count() >= 4); // root + 3 analysis subtasks
    }
}
