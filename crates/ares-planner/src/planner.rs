use ares_core::{
    new_id, AresError, Goal, Milestone, Plan, PlanDetails, PlanStatus, Task, TaskDependency,
    TaskStatus,
};
use ares_store::SqlitePlanRepository;
use ares_store::Store;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPlanOutput {
    pub goal: String,
    pub milestones: Vec<MockMilestoneOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockMilestoneOutput {
    pub title: String,
    pub description: Option<String>,
    pub tasks: Vec<MockTaskOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockTaskOutput {
    pub title: String,
    pub description: Option<String>,
    pub estimated_duration: Option<i32>, // in minutes
    pub complexity: Option<String>,      // "Low", "Medium", "High"
    pub dependencies: Vec<String>,       // titles of other tasks this depends on
}

#[async_trait]
pub trait PlannerProvider: Send + Sync {
    async fn generate_plan(&self, goal: &str) -> Result<MockPlanOutput, AresError>;
}

pub struct MockPlannerProvider;

#[async_trait]
impl PlannerProvider for MockPlannerProvider {
    async fn generate_plan(&self, goal: &str) -> Result<MockPlanOutput, AresError> {
        let goal_lower = goal.to_lowercase();

        if goal_lower.contains("oauth") || goal_lower.contains("authentication") {
            Ok(MockPlanOutput {
                goal: goal.to_string(),
                milestones: vec![
                    MockMilestoneOutput {
                        title: "Auth Foundation".to_string(),
                        description: Some("Set up the basic authentication layer and helper utilities".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Create auth module".to_string(),
                                description: Some("Initialize the authentication structures and config options in core".to_string()),
                                estimated_duration: Some(120),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec![],
                            },
                            MockTaskOutput {
                                title: "Add JWT middleware".to_string(),
                                description: Some("Implement JWT generation, verification, and HTTP middleware".to_string()),
                                estimated_duration: Some(180),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec!["Create auth module".to_string()],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: "OAuth Integration".to_string(),
                        description: Some("Integrate external OAuth providers (GitHub, Google)".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Configure GitHub OAuth provider".to_string(),
                                description: Some("Register API credentials and setup GitHub callback route".to_string()),
                                estimated_duration: Some(240),
                                complexity: Some("High".to_string()),
                                dependencies: vec!["Create auth module".to_string()],
                            },
                            MockTaskOutput {
                                title: "Configure Google OAuth provider".to_string(),
                                description: Some("Register API credentials and setup Google callback route".to_string()),
                                estimated_duration: Some(240),
                                complexity: Some("High".to_string()),
                                dependencies: vec!["Create auth module".to_string()],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: "UI and Testing".to_string(),
                        description: Some("Create login screens and perform end-to-end flow validation".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Create login button on dashboard".to_string(),
                                description: Some("Add login controls and style login portal page".to_string()),
                                estimated_duration: Some(60),
                                complexity: Some("Low".to_string()),
                                dependencies: vec![],
                            },
                            MockTaskOutput {
                                title: "Write integration tests for login flow".to_string(),
                                description: Some("Verify that users can login via mock OAuth flow and get valid JWTs".to_string()),
                                estimated_duration: Some(120),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec![
                                    "Add JWT middleware".to_string(),
                                    "Configure GitHub OAuth provider".to_string(),
                                ],
                            },
                        ],
                    },
                ],
            })
        } else if goal_lower.contains("redis") || goal_lower.contains("cache") {
            Ok(MockPlanOutput {
                goal: goal.to_string(),
                milestones: vec![
                    MockMilestoneOutput {
                        title: "Infrastructure Setup".to_string(),
                        description: Some("Provision the required Redis dependencies and local containers".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Add redis dependency to Cargo.toml".to_string(),
                                description: Some("Add redis-rs crate to workspace dependency list".to_string()),
                                estimated_duration: Some(30),
                                complexity: Some("Low".to_string()),
                                dependencies: vec![],
                            },
                            MockTaskOutput {
                                title: "Setup Redis Docker service".to_string(),
                                description: Some("Add redis service to docker-compose file for local developer environments".to_string()),
                                estimated_duration: Some(60),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec![],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: "Caching Layer implementation".to_string(),
                        description: Some("Create Redis client manager and implement db cache hooks".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Implement Redis connection pool client".to_string(),
                                description: Some("Create structured client with r2d2 connection pool support for Redis".to_string()),
                                estimated_duration: Some(120),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec!["Add redis dependency to Cargo.toml".to_string()],
                            },
                            MockTaskOutput {
                                title: "Add caching decorator/macro for database queries".to_string(),
                                description: Some("Cache hot queries (decisions, nodes) with key expiration support".to_string()),
                                estimated_duration: Some(240),
                                complexity: Some("High".to_string()),
                                dependencies: vec!["Implement Redis connection pool client".to_string()],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: "Validation".to_string(),
                        description: Some("Ensure database and Redis caches are properly synced".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Benchmark cache performance".to_string(),
                                description: Some("Create cache hit/miss instrumentation and verify load performance".to_string()),
                                estimated_duration: Some(120),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec!["Add caching decorator/macro for database queries".to_string()],
                            },
                        ],
                    },
                ],
            })
        } else if goal_lower.contains("mcp") || goal_lower.contains("refactor") {
            Ok(MockPlanOutput {
                goal: goal.to_string(),
                milestones: vec![
                    MockMilestoneOutput {
                        title: "Analysis & Separation".to_string(),
                        description: Some("Identify monolithic components and split handlers".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Audit current MCP handlers".to_string(),
                                description: Some("Inspect handlers in crates/ares-mcp and list structural patterns".to_string()),
                                estimated_duration: Some(90),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec![],
                            },
                            MockTaskOutput {
                                title: "Split handlers into dedicated files".to_string(),
                                description: Some("Move tool definitions and handlings into distinct submodule files".to_string()),
                                estimated_duration: Some(180),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec!["Audit current MCP handlers".to_string()],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: "Abstraction Layer".to_string(),
                        description: Some("Create server trait and generic router mapping".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Introduce MCPServer trait".to_string(),
                                description: Some("Establish a standard trait mapping request/response flows for MCP servers".to_string()),
                                estimated_duration: Some(240),
                                complexity: Some("High".to_string()),
                                dependencies: vec!["Split handlers into dedicated files".to_string()],
                            },
                            MockTaskOutput {
                                title: "Implement unified tool registration helper".to_string(),
                                description: Some("Create macro or builder pattern to register schema JSON easily".to_string()),
                                estimated_duration: Some(120),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec!["Introduce MCPServer trait".to_string()],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: "Clean Up".to_string(),
                        description: Some("Remove old unused files and format".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Remove legacy handler routes".to_string(),
                                description: Some("Delete old router logic and deprecated comments".to_string()),
                                estimated_duration: Some(60),
                                complexity: Some("Low".to_string()),
                                dependencies: vec!["Implement unified tool registration helper".to_string()],
                            },
                        ],
                    },
                ],
            })
        } else if goal_lower.contains("telemetry") || goal_lower.contains("bug") {
            Ok(MockPlanOutput {
                goal: goal.to_string(),
                milestones: vec![
                    MockMilestoneOutput {
                        title: "Investigation".to_string(),
                        description: Some("Locate the telemetry connection issue".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Reproduce telemetry offline error in test".to_string(),
                                description: Some("Write unit test that asserts telemetry state mapping correctness".to_string()),
                                estimated_duration: Some(90),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec![],
                            },
                            MockTaskOutput {
                                title: "Inspect store telemetry queries".to_string(),
                                description: Some("Check DB schema constraints on telemetry tables".to_string()),
                                estimated_duration: Some(90),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec![],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: "Fix".to_string(),
                        description: Some("Apply fixes to telemetry database schema and mapping".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Update telemetry repository SQL schema to support default timestamp".to_string(),
                                description: Some("Set auto default timestamps on schema insertion points".to_string()),
                                estimated_duration: Some(60),
                                complexity: Some("Low".to_string()),
                                dependencies: vec!["Inspect store telemetry queries".to_string()],
                            },
                            MockTaskOutput {
                                title: "Fix front-end mapping of offline telemetry state".to_string(),
                                description: Some("Ensure UI renders live connection status properly".to_string()),
                                estimated_duration: Some(120),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec!["Update telemetry repository SQL schema to support default timestamp".to_string()],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: "Verification".to_string(),
                        description: Some("Validate telemetries are reported correctly".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: "Run telemetry benchmark tests".to_string(),
                                description: Some("Trigger benchmark engine and inspect live data report".to_string()),
                                estimated_duration: Some(60),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec!["Update telemetry repository SQL schema to support default timestamp".to_string()],
                            },
                        ],
                    },
                ],
            })
        } else {
            // Generic Fallback Plan
            Ok(MockPlanOutput {
                goal: goal.to_string(),
                milestones: vec![
                    MockMilestoneOutput {
                        title: format!("Research & Design for {}", goal),
                        description: Some(format!("Prepare architecture and audit current resources for {}", goal)),
                        tasks: vec![
                            MockTaskOutput {
                                title: format!("Research requirements for {}", goal),
                                description: Some("Compile developer documents and list edge cases".to_string()),
                                estimated_duration: Some(120),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec![],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: format!("Implementation of {}", goal),
                        description: Some(format!("Implement the core changes requested by {}", goal)),
                        tasks: vec![
                            MockTaskOutput {
                                title: format!("Develop core logic for {}", goal),
                                description: Some("Create services, helper methods, and structs".to_string()),
                                estimated_duration: Some(240),
                                complexity: Some("High".to_string()),
                                dependencies: vec![format!("Research requirements for {}", goal)],
                            },
                            MockTaskOutput {
                                title: format!("Integrate {} into application", goal),
                                description: Some("Configure API gateways, store integrations, and settings configurations".to_string()),
                                estimated_duration: Some(180),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec![format!("Develop core logic for {}", goal)],
                            },
                        ],
                    },
                    MockMilestoneOutput {
                        title: "Testing".to_string(),
                        description: Some("Verify logic using unit and integration tests".to_string()),
                        tasks: vec![
                            MockTaskOutput {
                                title: format!("Verify {} with unit tests", goal),
                                description: Some("Write test cases with mock values and verify assertions".to_string()),
                                estimated_duration: Some(120),
                                complexity: Some("Medium".to_string()),
                                dependencies: vec![format!("Integrate {} into application", goal)],
                            },
                        ],
                    },
                ],
            })
        }
    }
}

pub struct PlannerEngine {
    provider: Box<dyn PlannerProvider>,
    store: Store,
}

impl PlannerEngine {
    pub fn new(store: Store, provider: Box<dyn PlannerProvider>) -> Self {
        Self { provider, store }
    }

    pub async fn create_plan_from_goal(
        &self,
        goal_title: &str,
        priority: &str,
    ) -> Result<PlanDetails, AresError> {
        info!(goal = %goal_title, "Starting autonomous plan generation");

        let repo = SqlitePlanRepository::new(self.store.clone());

        // 1. Save Goal
        let goal_id = format!("goal_{}", new_id());
        let goal = Goal {
            id: goal_id.clone(),
            title: goal_title.to_string(),
            description: Some(format!(
                "Automatically generated plan for goal: {}",
                goal_title
            )),
            priority: priority.to_string(),
            deadline: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        repo.create_goal(&goal)?;

        // 2. Generate plan structured response from the provider
        let output = self.provider.generate_plan(goal_title).await?;

        // 3. Save Plan
        let plan_id = format!("plan_{}", new_id());
        let plan = Plan {
            id: plan_id.clone(),
            goal_id: goal_id.clone(),
            state: PlanStatus::Generated,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        repo.create_plan(&plan)?;

        // 4. Flatten all tasks to topological sort them globally
        let mut flat_tasks = Vec::new();
        // Maps task title (lowercase) to temporary tuple (MilestoneId, MockTaskOutput)
        let mut milestone_task_mappings = Vec::new();

        for mock_milestone in &output.milestones {
            let milestone_id = format!("ms_{}", new_id());
            let milestone = Milestone {
                id: milestone_id.clone(),
                plan_id: plan_id.clone(),
                title: mock_milestone.title.clone(),
                description: mock_milestone.description.clone(),
                created_at: Utc::now(),
            };

            repo.create_milestone(&milestone)?;

            for mock_task in &mock_milestone.tasks {
                flat_tasks.push(mock_task.clone());
                milestone_task_mappings.push((milestone_id.clone(), mock_task.clone()));
            }
        }

        // 5. Run Topological Sort to determine execution order
        let sorted_indices = topological_sort(&flat_tasks);

        // Map task title (lowercase) -> TaskId (for database dependency mapping)
        let mut task_title_to_id = HashMap::new();
        let mut task_records = Vec::new();

        // Initialize task records with IDs
        for (milestone_id, mock_task) in &milestone_task_mappings {
            let task_id = format!("task_{}", new_id());
            task_title_to_id.insert(
                mock_task.title.to_lowercase().trim().to_string(),
                task_id.clone(),
            );
            task_records.push((task_id, milestone_id.clone(), mock_task.clone()));
        }

        // Save tasks in order with execution_order
        let mut saved_tasks = Vec::new();
        for (order, &idx) in sorted_indices.iter().enumerate() {
            let (task_id, milestone_id, mock_task) = &task_records[idx];
            let execution_order = (order + 1) as i32;

            let task = Task {
                id: task_id.clone(),
                milestone_id: Some(milestone_id.clone()),
                plan_id: plan_id.clone(),
                title: mock_task.title.clone(),
                description: mock_task.description.clone(),
                status: TaskStatus::Pending,
                estimated_duration: mock_task.estimated_duration,
                complexity: mock_task.complexity.clone(),
                execution_order,
                created_at: Utc::now(),
            };

            repo.create_task(&task)?;
            saved_tasks.push(task);
        }

        // Save dependencies
        let mut saved_dependencies = Vec::new();
        for (task_id, _, mock_task) in &task_records {
            for dep_title in &mock_task.dependencies {
                let dep_title_clean = dep_title.to_lowercase().trim().to_string();
                if let Some(dep_task_id) = task_title_to_id.get(&dep_title_clean) {
                    let dep = TaskDependency {
                        task_id: task_id.clone(),
                        depends_on_id: dep_task_id.clone(),
                    };
                    repo.create_task_dependency(&dep)?;
                    saved_dependencies.push(dep);
                }
            }
        }

        // Retrieve saved milestones
        let saved_milestones = repo.get_milestones_for_plan(&plan_id)?;

        Ok(PlanDetails {
            plan,
            goal,
            milestones: saved_milestones,
            tasks: saved_tasks,
            dependencies: saved_dependencies,
        })
    }
}

pub fn topological_sort(tasks: &[MockTaskOutput]) -> Vec<usize> {
    let mut adjacency: HashMap<String, Vec<usize>> = HashMap::new();
    let mut in_degree = vec![0; tasks.len()];

    // Map of task title (lowercase) to its index
    let title_to_index: HashMap<String, usize> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.title.to_lowercase().trim().to_string(), i))
        .collect();

    for (i, task) in tasks.iter().enumerate() {
        for dep in &task.dependencies {
            let dep_clean = dep.to_lowercase().trim().to_string();
            if let Some(&_dep_idx) = title_to_index.get(&dep_clean) {
                adjacency.entry(dep_clean).or_default().push(i);
                in_degree[i] += 1;
            }
        }
    }

    let mut queue = VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(i);
        }
    }

    let mut result = Vec::new();
    while let Some(u) = queue.pop_front() {
        result.push(u);
        let title = tasks[u].title.to_lowercase().trim().to_string();
        if let Some(neighbors) = adjacency.get(&title) {
            for &v in neighbors {
                in_degree[v] -= 1;
                if in_degree[v] == 0 {
                    queue.push_back(v);
                }
            }
        }
    }

    // Fallback: If there is a cycle or disconnected component, add remaining indices
    if result.len() < tasks.len() {
        let visited: HashSet<usize> = result.iter().cloned().collect();
        for i in 0..tasks.len() {
            if !visited.contains(&i) {
                result.push(i);
            }
        }
    }

    result
}
