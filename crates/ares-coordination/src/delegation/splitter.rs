use serde::{Deserialize, Serialize};

/// How a task should be split for delegation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitStrategy {
    /// Sub-tasks must run in sequence.
    Sequential,
    /// Sub-tasks can run in parallel.
    Parallel,
    /// Sub-tasks form a pipeline (output of one is input to next).
    Pipeline,
    /// Task cannot be meaningfully split.
    Unsplittable,
}

/// A sub-task created by splitting a parent task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub name: String,
    pub description: String,
    pub estimated_complexity: f64,
    pub dependencies: Vec<usize>, // Indices of other sub-tasks this depends on
}

/// Analyzes task complexity and determines optimal decomposition.
pub struct TaskSplitter;

impl TaskSplitter {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a task and suggest how to split it.
    pub fn analyze(&self, description: &str, complexity: f64) -> SplitResult {
        if complexity < 0.3 {
            return SplitResult {
                strategy: SplitStrategy::Unsplittable,
                sub_tasks: vec![SubTask {
                    name: "original".into(),
                    description: description.to_string(),
                    estimated_complexity: complexity,
                    dependencies: vec![],
                }],
                rationale: "Task is simple enough for a single agent".into(),
            };
        }

        // Determine strategy based on complexity
        let (strategy, sub_tasks) = if complexity > 0.8 {
            // Complex task — split into pipeline
            let tasks = vec![
                SubTask {
                    name: "analysis".into(),
                    description: format!("Analyze requirements for: {}", description),
                    estimated_complexity: complexity * 0.2,
                    dependencies: vec![],
                },
                SubTask {
                    name: "implementation".into(),
                    description: format!("Implement solution for: {}", description),
                    estimated_complexity: complexity * 0.5,
                    dependencies: vec![0],
                },
                SubTask {
                    name: "verification".into(),
                    description: format!("Verify solution for: {}", description),
                    estimated_complexity: complexity * 0.3,
                    dependencies: vec![1],
                },
            ];
            (SplitStrategy::Pipeline, tasks)
        } else if complexity > 0.5 {
            // Medium complexity — parallel split
            let tasks = vec![
                SubTask {
                    name: "part_a".into(),
                    description: format!("First component of: {}", description),
                    estimated_complexity: complexity * 0.5,
                    dependencies: vec![],
                },
                SubTask {
                    name: "part_b".into(),
                    description: format!("Second component of: {}", description),
                    estimated_complexity: complexity * 0.5,
                    dependencies: vec![],
                },
            ];
            (SplitStrategy::Parallel, tasks)
        } else {
            // Light complexity — sequential
            let tasks = vec![
                SubTask {
                    name: "prepare".into(),
                    description: format!("Prepare for: {}", description),
                    estimated_complexity: complexity * 0.3,
                    dependencies: vec![],
                },
                SubTask {
                    name: "execute".into(),
                    description: format!("Execute: {}", description),
                    estimated_complexity: complexity * 0.7,
                    dependencies: vec![0],
                },
            ];
            (SplitStrategy::Sequential, tasks)
        };

        SplitResult {
            strategy,
            sub_tasks,
            rationale: format!("Task complexity {:.2} suggests decomposition", complexity),
        }
    }
}

impl Default for TaskSplitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of task splitting analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitResult {
    pub strategy: SplitStrategy,
    pub sub_tasks: Vec<SubTask>,
    pub rationale: String,
}
