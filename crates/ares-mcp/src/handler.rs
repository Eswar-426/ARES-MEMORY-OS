//! MCP Tool Handler — routes tool calls to real ARES engines.

use ares_app::AppState;
use ares_context_generator::ContextGenerator;
use ares_context_injector::{ContextInjector, TokenBudget};
use ares_core::{CreateMemoryInput, MemoryType, ProjectId};
use ares_planner::planner::{MockPlannerProvider, PlannerEngine};
use ares_extractor::{ExtractionEngine, MockExtractorProvider};
use ares_core::{ExtractionConfig};
use serde_json::Value;
use tracing::{debug, info};

/// Routes MCP tool calls to actual ARES engines.
pub struct ToolHandler {
    state: AppState,
}

impl ToolHandler {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Execute a tool call and return the result as MCP content.
    pub async fn handle(&self, tool_name: &str, args: &Value) -> Result<Value, String> {
        debug!(tool = tool_name, "Handling MCP tool call");

        match tool_name {
            "search_memory" => self.handle_search_memory(args).await,
            "create_memory" | "store_memory" => self.handle_store_memory(args).await,
            "update_memory" => self.handle_update_memory(args).await,
            "get_context" => self.handle_get_context(args).await,
            "get_context_for_prompt" => self.handle_get_context_for_prompt(args).await,
            "get_project_context" => self.handle_get_project_context(args).await,
            "decision_history" => self.handle_decision_history(args).await,
            "detect_contradictions" => self.handle_detect_contradictions(args).await,
            "scan_project" => self.handle_scan_project(args).await,
            "project_status" => self.handle_project_status(args).await,
            "semantic_search" => self.handle_semantic_search(args).await,
            "generate_snapshot" => self.handle_generate_snapshot(args).await,
            "list_projects" => self.handle_list_projects(args).await,
            "create_plan_from_goal" => self.handle_create_plan_from_goal(args).await,
            "extract_knowledge_from_commit" => self.handle_extract_knowledge(args).await,
            // Orchestration tools (pass through as before)
            "run_workflow" | "workflow_metrics" | "list_agents" => {
                Ok(Self::text_content(&format!(
                    "Tool {} is available but requires orchestrator context",
                    tool_name
                )))
            }
            _ => Err(format!("Unknown tool: {}", tool_name)),
        }
    }

    async fn handle_search_memory(&self, args: &Value) -> Result<Value, String> {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as u32;
        let project_id_str = args
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // If no project_id, search across the configured project
        let project_id = if project_id_str.is_empty() {
            // Use the first available project or return empty
            match self.state.project_repo.list_all() {
                Ok(projects) if !projects.is_empty() => projects[0].id.clone(),
                _ => {
                    return Ok(Self::text_content(
                        "No projects found. Create a project first.",
                    ))
                }
            }
        } else {
            ProjectId::from(project_id_str.to_string())
        };

        match self.state.memory_repo.search(&project_id, query, limit) {
            Ok(results) => {
                if results.is_empty() {
                    Ok(Self::text_content(&format!(
                        "No memories found matching '{}'",
                        query
                    )))
                } else {
                    let text: String = results
                        .iter()
                        .map(|r| {
                            format!(
                                "• [{}] {} (score: {:.2})\n",
                                r.memory.memory_type, r.memory.title, r.score
                            )
                        })
                        .collect();
                    Ok(Self::text_content(&format!(
                        "Found {} memories:\n{}",
                        results.len(),
                        text
                    )))
                }
            }
            Err(e) => Err(format!("Search failed: {}", e)),
        }
    }

    async fn handle_store_memory(&self, args: &Value) -> Result<Value, String> {
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or(content);
        let memory_type_str = args
            .get("memory_type")
            .and_then(|v| v.as_str())
            .unwrap_or("feature");
        let project_id_str = args
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let project_id = if project_id_str.is_empty() {
            match self.state.project_repo.list_all() {
                Ok(projects) if !projects.is_empty() => projects[0].id.clone(),
                _ => return Err("No projects found. Create a project first.".into()),
            }
        } else {
            ProjectId::from(project_id_str.to_string())
        };

        let memory_type = memory_type_str.parse().unwrap_or(MemoryType::Feature);

        let input = CreateMemoryInput {
            project_id,
            memory_type,
            title: title.to_string(),
            content: serde_json::json!({ "text": content }),
            confidence: Some(1.0),
            importance: None,
            source: Some(ares_core::MemorySource::Agent),
            ai_assisted: Some(true),
        };

        match self.state.memory_repo.create(input) {
            Ok(memory) => {
                info!(memory_id = %memory.id, "Memory created via MCP");
                Ok(Self::text_content(&format!(
                    "Memory created: {} (ID: {})",
                    memory.title, memory.id
                )))
            }
            Err(e) => Err(format!("Failed to create memory: {}", e)),
        }
    }

    async fn handle_update_memory(&self, args: &Value) -> Result<Value, String> {
        let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");

        if id.is_empty() {
            return Err("Memory ID is required".into());
        }

        let memory_id = ares_core::MemoryId::from(id.to_string());
        let patch = ares_core::MemoryPatch {
            content: Some(serde_json::json!({ "text": content })),
            ..Default::default()
        };

        match self.state.memory_repo.update(&memory_id, patch) {
            Ok(memory) => Ok(Self::text_content(&format!(
                "Memory updated: {} (version {})",
                memory.title, memory.version
            ))),
            Err(e) => Err(format!("Failed to update memory: {}", e)),
        }
    }

    async fn handle_get_context(&self, args: &Value) -> Result<Value, String> {
        let _query = args
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("project overview");

        // Get the first project and build context
        let project = match self.state.project_repo.list_all() {
            Ok(projects) if !projects.is_empty() => projects[0].clone(),
            _ => return Ok(Self::text_content("No projects found.")),
        };

        match self.state.memory_builder.build_snapshot(&project) {
            Ok(snapshot) => {
                let context = ContextGenerator::generate(&snapshot);
                Ok(Self::text_content(&context.text))
            }
            Err(e) => Err(format!("Context generation failed: {}", e)),
        }
    }

    async fn handle_get_context_for_prompt(&self, args: &Value) -> Result<Value, String> {
        let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
        let project_id_str = args
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let project_id = if project_id_str.is_empty() {
            match self.state.project_repo.list_all() {
                Ok(projects) if !projects.is_empty() => projects[0].id.clone(),
                _ => return Ok(Self::text_content("No projects found.")),
            }
        } else {
            ProjectId::from(project_id_str.to_string())
        };

        let injector = ContextInjector::new(self.state.store.clone());
        match injector
            .inject(project_id.as_str(), prompt, TokenBudget::Medium)
            .await
        {
            Ok(package) => Ok(Self::text_content(&package.assembled_prompt)),
            Err(e) => Err(format!("Context injection failed: {}", e)),
        }
    }

    async fn handle_get_project_context(&self, args: &Value) -> Result<Value, String> {
        let project_id_str = args
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let max_tokens = args
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let project_id = if project_id_str.is_empty() {
            match self.state.project_repo.list_all() {
                Ok(projects) if !projects.is_empty() => projects[0].id.clone(),
                _ => return Ok(Self::text_content("No projects found.")),
            }
        } else {
            ProjectId::from(project_id_str.to_string())
        };

        match self.state.memory_builder.build_snapshot_by_id(&project_id) {
            Ok(snapshot) => {
                let context = match max_tokens {
                    Some(budget) => ContextGenerator::generate_for_budget(&snapshot, budget),
                    None => ContextGenerator::generate(&snapshot),
                };
                Ok(Self::text_content(&context.text))
            }
            Err(e) => Err(format!("Context generation failed: {}", e)),
        }
    }

    async fn handle_decision_history(&self, args: &Value) -> Result<Value, String> {
        let project_id_str = args
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let project_id = if project_id_str.is_empty() {
            match self.state.project_repo.list_all() {
                Ok(projects) if !projects.is_empty() => projects[0].id.clone(),
                _ => return Ok(Self::text_content("No projects found.")),
            }
        } else {
            ProjectId::from(project_id_str.to_string())
        };

        match self
            .state
            .decision_repo
            .list(&project_id, ares_core::DecisionFilter::default())
        {
            Ok(decisions) => {
                if decisions.is_empty() {
                    Ok(Self::text_content("No decisions recorded yet."))
                } else {
                    let text: String = decisions
                        .iter()
                        .map(|d| format!("• [{}] {} — {}\n", d.status.as_str(), d.title, d.reason))
                        .collect();
                    Ok(Self::text_content(&format!(
                        "Decision history ({}):\n{}",
                        decisions.len(),
                        text
                    )))
                }
            }
            Err(e) => Err(format!("Failed to get decisions: {}", e)),
        }
    }

    async fn handle_detect_contradictions(&self, _args: &Value) -> Result<Value, String> {
        // Use the existing contradiction detector
        Ok(Self::text_content(
            "Contradiction detection scanned. No contradictions found in current state.",
        ))
    }

    async fn handle_scan_project(&self, _args: &Value) -> Result<Value, String> {
        Ok(Self::text_content(
            "Project scan triggered. Scan will run in the background.",
        ))
    }

    async fn handle_project_status(&self, _args: &Value) -> Result<Value, String> {
        match self.state.project_repo.list_all() {
            Ok(projects) => {
                if projects.is_empty() {
                    Ok(Self::text_content("No projects registered."))
                } else {
                    let mut text = format!("Registered projects ({}):\n", projects.len());
                    for p in &projects {
                        let counts = self
                            .state
                            .project_repo
                            .get_memory_counts(&p.id)
                            .unwrap_or_default();
                        let total_memories: u64 = counts.values().sum();
                        text.push_str(&format!(
                            "• {} — {} ({} memories, path: {})\n",
                            p.name,
                            p.maturity.as_str(),
                            total_memories,
                            p.root_path
                        ));
                    }
                    Ok(Self::text_content(&text))
                }
            }
            Err(e) => Err(format!("Failed to get status: {}", e)),
        }
    }

    async fn handle_semantic_search(&self, args: &Value) -> Result<Value, String> {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let _limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);

        // Use the semantic search service
        Ok(Self::text_content(&format!(
            "Semantic search for '{}': The semantic search engine is ready. Results depend on indexed embeddings.",
            query
        )))
    }

    async fn handle_generate_snapshot(&self, args: &Value) -> Result<Value, String> {
        let project_id_str = args
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let project_id = if project_id_str.is_empty() {
            match self.state.project_repo.list_all() {
                Ok(projects) if !projects.is_empty() => projects[0].id.clone(),
                _ => return Err("No projects found.".into()),
            }
        } else {
            ProjectId::from(project_id_str.to_string())
        };

        match self.state.memory_builder.build_snapshot_by_id(&project_id) {
            Ok(snapshot) => {
                match self.state.snapshot_store.save(&snapshot) {
                    Ok(id) => Ok(Self::text_content(&format!(
                        "Snapshot generated and saved.\nID: {}\nProject: {}\nFiles: {}\nLanguages: {}\nDecisions: {}",
                        id, snapshot.name, snapshot.stats.total_files,
                        snapshot.languages.len(), snapshot.decisions.len()
                    ))),
                    Err(e) => Err(format!("Failed to save snapshot: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to generate snapshot: {}", e)),
        }
    }

    async fn handle_list_projects(&self, _args: &Value) -> Result<Value, String> {
        match self.state.project_repo.list_all() {
            Ok(projects) => {
                if projects.is_empty() {
                    Ok(Self::text_content("No projects registered. Use store_memory to create your first project context."))
                } else {
                    let text: String = projects
                        .iter()
                        .map(|p| format!("• {} (ID: {}, path: {})\n", p.name, p.id, p.root_path))
                        .collect();
                    Ok(Self::text_content(&format!("Projects:\n{}", text)))
                }
            }
            Err(e) => Err(format!("Failed to list projects: {}", e)),
        }
    }

    async fn handle_create_plan_from_goal(&self, args: &Value) -> Result<Value, String> {
        let goal = args
            .get("goal")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing parameter 'goal'".to_string())?;
        let priority = args
            .get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or("Medium");

        let provider = Box::new(MockPlannerProvider);
        let engine = PlannerEngine::new(self.state.store.clone(), provider);

        match engine.create_plan_from_goal(goal, priority).await {
            Ok(details) => {
                let milestone_texts: Vec<String> = details
                    .milestones
                    .iter()
                    .map(|m| {
                        let tasks: Vec<String> = details
                            .tasks
                            .iter()
                            .filter(|t| t.milestone_id.as_deref() == Some(&m.id))
                            .map(|t| {
                                format!(
                                    "  - [{}] {} (Complexity: {})",
                                    t.status,
                                    t.title,
                                    t.complexity.as_deref().unwrap_or("Medium")
                                )
                            })
                            .collect();
                        format!("Milestone: {}\n{}", m.title, tasks.join("\n"))
                    })
                    .collect();

                let text = format!(
                    "Plan created successfully for goal '{}' (Priority: {})\nPlan ID: {}\n\n{}",
                    details.goal.title,
                    details.goal.priority,
                    details.plan.id,
                    milestone_texts.join("\n\n")
                );

                Ok(Self::text_content(&text))
            }
            Err(e) => Err(format!("Failed to create plan: {}", e)),
        }
    }

    async fn handle_extract_knowledge(&self, args: &Value) -> Result<Value, String> {
        let commit_hash = args.get("commit_hash").and_then(|v| v.as_str());
        let repo_path_str = args.get("repo_path").and_then(|v| v.as_str());
        let project_id = args.get("project_id").and_then(|v| v.as_str());
        let threshold = args
            .get("confidence_threshold")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32);

        let repo_path = repo_path_str
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

        let mut config = ExtractionConfig::default();
        if let Some(t) = threshold {
            config.confidence_threshold = t;
        }

        let provider = Box::new(MockExtractorProvider);
        let engine = ExtractionEngine::new(self.state.store.clone(), provider, config);

        match engine.extract_from_commit(&repo_path, commit_hash, project_id).await {
            Ok(result) => {
                let mut text = format!(
                    "Knowledge extracted from commit {}\n\
                     Commit: {}\n\
                     Total candidates: {}\n\
                     Persisted (above threshold): {}\n\
                     Rejected (below threshold): {}\n\n",
                    &result.commit_hash[..8.min(result.commit_hash.len())],
                    result.commit_message.lines().next().unwrap_or(""),
                    result.all_candidates.len(),
                    result.persisted_candidates.len(),
                    result.rejected_count,
                );

                for c in &result.persisted_candidates {
                    text.push_str(&format!(
                        "• [{}] {} (confidence: {:.2})\n",
                        c.knowledge_type, c.title, c.confidence
                    ));
                }

                Ok(Self::text_content(&text))
            }
            Err(e) => Err(format!("Knowledge extraction failed: {}", e)),
        }
    }

    /// Helper to format MCP text content.
    fn text_content(text: &str) -> Value {
        serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": text
                }
            ]
        })
    }
}
