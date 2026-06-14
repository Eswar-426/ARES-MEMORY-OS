#[derive(Debug, Clone, PartialEq)]
pub enum KeywordCategory {
    Architecture,
    Requirements,
    Features,
    Decisions,
    BugHistory,
    Recovery,
}

#[derive(Debug, Clone)]
pub struct ExpectedKeyword {
    pub word: String,
    pub category: KeywordCategory,
}

#[derive(Debug, Clone)]
pub struct ChainStep {
    pub description: String,
    pub prompt: String,
    pub expected_keywords: Vec<ExpectedKeyword>,
}

#[derive(Debug, Clone)]
pub struct ContinuityScenario {
    pub id: String,
    pub name: String,
    pub steps: Vec<ChainStep>,
}

pub fn get_test_scenarios() -> Vec<ContinuityScenario> {
    vec![
        // Scenario 1: Build REST API
        ContinuityScenario {
            id: "scenario_1".to_string(),
            name: "Build REST API (Arch -> Feature -> Bug)".to_string(),
            steps: vec![
                ChainStep {
                    description: "Initial Architecture".to_string(),
                    prompt: "Create an Axum REST API with PostgreSQL for a User Service. Use CQRS pattern. Output all files using the format: ```file:path=filename\\n...```".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "Axum".to_string(), category: KeywordCategory::Architecture },
                        ExpectedKeyword { word: "PostgreSQL".to_string(), category: KeywordCategory::Decisions },
                        ExpectedKeyword { word: "CQRS".to_string(), category: KeywordCategory::Architecture },
                    ],
                },
                ChainStep {
                    description: "Feature Addition".to_string(),
                    prompt: "Add a feature to register a new user. Keep in mind the existing CQRS pattern and Postgres DB. Output all files using the format: ```file:path=filename\\n...```".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "register".to_string(), category: KeywordCategory::Features },
                        ExpectedKeyword { word: "CQRS".to_string(), category: KeywordCategory::Architecture },
                        ExpectedKeyword { word: "Postgres".to_string(), category: KeywordCategory::Requirements },
                    ],
                },
                ChainStep {
                    description: "Bug Fix & Recovery".to_string(),
                    prompt: "There's a bug: registering an existing email crashes the server. Fix it. Output all files using the format: ```file:path=filename\\n...```".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "email".to_string(), category: KeywordCategory::BugHistory },
                        ExpectedKeyword { word: "crash".to_string(), category: KeywordCategory::BugHistory },
                        ExpectedKeyword { word: "conflict".to_string(), category: KeywordCategory::Recovery },
                        ExpectedKeyword { word: "Postgres".to_string(), category: KeywordCategory::Recovery },
                    ],
                },
            ],
        },
        // Scenario 2: Rust Workspace
        ContinuityScenario {
            id: "scenario_2".to_string(),
            name: "Rust Workspace Evolution".to_string(),
            steps: vec![
                ChainStep {
                    description: "Setup".to_string(),
                    prompt: "Create a Rust workspace with two crates: `core` and `api`. Output all files using the format: ```file:path=filename\\n...```".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "workspace".to_string(), category: KeywordCategory::Architecture },
                        ExpectedKeyword { word: "core".to_string(), category: KeywordCategory::Features },
                        ExpectedKeyword { word: "api".to_string(), category: KeywordCategory::Features },
                    ],
                },
                ChainStep {
                    description: "Add Feature".to_string(),
                    prompt: "Add a shared utilities module in the `core` crate that the `api` crate uses. Output all files using the format: ```file:path=filename\\n...```".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "utilities".to_string(), category: KeywordCategory::Features },
                        ExpectedKeyword { word: "core".to_string(), category: KeywordCategory::Recovery },
                    ],
                },
            ]
        },
        // Scenario 3: React Dashboard
        ContinuityScenario {
            id: "scenario_3".to_string(),
            name: "React Dashboard MVP".to_string(),
            steps: vec![
                ChainStep {
                    description: "Initialize".to_string(),
                    prompt: "Create a React + Vite dashboard. Use TypeScript and Tailwind CSS.".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "Vite".to_string(), category: KeywordCategory::Architecture },
                        ExpectedKeyword { word: "TypeScript".to_string(), category: KeywordCategory::Requirements },
                        ExpectedKeyword { word: "Tailwind".to_string(), category: KeywordCategory::Requirements },
                    ],
                },
                ChainStep {
                    description: "Components".to_string(),
                    prompt: "Add a Sidebar component with Glassmorphism styling.".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "Sidebar".to_string(), category: KeywordCategory::Features },
                        ExpectedKeyword { word: "Glassmorphism".to_string(), category: KeywordCategory::Decisions },
                    ],
                },
                ChainStep {
                    description: "Fixing State".to_string(),
                    prompt: "The Sidebar state isn't persisting across routes. Fix it using React Context.".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "Context".to_string(), category: KeywordCategory::Decisions },
                        ExpectedKeyword { word: "Sidebar".to_string(), category: KeywordCategory::BugHistory },
                    ],
                },
            ]
        },
        // Scenario 4: Microservice Migration
        ContinuityScenario {
            id: "scenario_4".to_string(),
            name: "Microservice Migration".to_string(),
            steps: vec![
                ChainStep {
                    description: "Monolith".to_string(),
                    prompt: "We have a Node.js monolith doing Payments and Auth. We decided to split Payments into a Go microservice.".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "Node".to_string(), category: KeywordCategory::Architecture },
                        ExpectedKeyword { word: "Go".to_string(), category: KeywordCategory::Decisions },
                        ExpectedKeyword { word: "Payments".to_string(), category: KeywordCategory::Requirements },
                    ],
                },
                ChainStep {
                    description: "Migration Step".to_string(),
                    prompt: "Implement gRPC communication between the Node Auth service and the new Go Payments service.".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "gRPC".to_string(), category: KeywordCategory::Architecture },
                        ExpectedKeyword { word: "Auth".to_string(), category: KeywordCategory::Recovery },
                    ],
                },
            ]
        },
        // Scenario 5: Large AI Agent System
        ContinuityScenario {
            id: "scenario_5".to_string(),
            name: "Large AI Agent System".to_string(),
            steps: vec![
                ChainStep {
                    description: "Core Engine".to_string(),
                    prompt: "Build an AI Agent System that uses a Swarm architecture with a Governor node.".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "Swarm".to_string(), category: KeywordCategory::Architecture },
                        ExpectedKeyword { word: "Governor".to_string(), category: KeywordCategory::Features },
                    ],
                },
                ChainStep {
                    description: "Memory Subsystem".to_string(),
                    prompt: "Integrate a Project Memory graph into the agents so they don't lose context.".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "Memory".to_string(), category: KeywordCategory::Requirements },
                        ExpectedKeyword { word: "Graph".to_string(), category: KeywordCategory::Decisions },
                    ],
                },
                ChainStep {
                    description: "Bug Fix".to_string(),
                    prompt: "Agents are hallucinating context from old sessions. Implement an episodic memory decay function.".to_string(),
                    expected_keywords: vec![
                        ExpectedKeyword { word: "decay".to_string(), category: KeywordCategory::Features },
                        ExpectedKeyword { word: "hallucinating".to_string(), category: KeywordCategory::BugHistory },
                    ],
                },
            ]
        },
    ]
}
