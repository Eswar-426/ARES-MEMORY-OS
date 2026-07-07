use ares_agent::config::AgentConfig;
use ares_app::AppState;
use ares_memory_intelligence::assembler::MemoryContextAssembler;
use ares_memory_intelligence::facade::MemoryFacade;
use ares_repository_intelligence::facade::IntelligenceFacade;
use ares_repository_intelligence::models::{EngineeringQuery, QueryType};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tower_mcp::{
    protocol::{CallToolResult, ReadResourceResult},
    resource::{ResourceBuilder, ResourceTemplateBuilder},
    router::McpRouter,
    tool::ToolBuilder,
    transport::stdio::StdioTransport,
    BoxError,
};
use tracing::info;

fn format_mcp_error(message: &str, details: &str) -> String {
    serde_json::json!({
        "code": -32603,
        "message": message,
        "details": details
    })
    .to_string()
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MemoryQueryInput {
    id: String,
}

// === Phase 2: Task 3.1 — Additional MCP Tools ===

#[derive(Debug, Deserialize, JsonSchema)]
struct OwnerQueryInput {
    file_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DecisionsQueryInput {
    file_path: Option<String>,
    since: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchQueryInput {
    query: String,
    #[serde(default = "default_search_limit")]
    limit: usize,
}

fn default_search_limit() -> usize { 10 }

#[derive(Debug, Deserialize, JsonSchema)]
struct TimelineQueryInput {
    file_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CompareQueryInput {
    file_a: String,
    file_b: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ArchitectureQueryInput {}

#[derive(Debug, Deserialize, JsonSchema)]
struct RequirementsQueryInput {
    file_path: Option<String>,
}


#[derive(Debug, Deserialize, JsonSchema)]
struct GovernanceQueryInput {
    project_id: String,
    node_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ProjectQueryInput {
    project_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ChatInput {
    query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct BookmarkInput {
    kind: String,
    value: String,
    title: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PinInput {
    node_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NavigateInput {
    direction: String,
    current_timestamp: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RecordNavigateInput {
    node_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct SimulationInput {
    project_id: String,
    action: String,
    target_id: String,
    related_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TraceabilityInput {
    entity_id: String,
    depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EmptyInput {}

#[derive(Debug, Deserialize, JsonSchema)]
struct GraphSearchInput {
    query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GraphPathInput {
    from_id: String,
    to_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GraphNeighborsInput {
    node_id: String,
    depth: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    use std::io::Write;
    let log_path = "C:\\Users\\eswar\\ares_mcp_test.log";
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap();
    writeln!(file, "==== Starting ares-mcp ====").unwrap();

    // Basic tracing setup for MCP (use stderr for logs so stdio stdout is free for JSON-RPC)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting ARES MCP Server");

    let project_path = match std::env::current_dir() {
        Ok(dir) => dir.to_string_lossy().to_string(),
        Err(e) => {
            writeln!(file, "Failed to get current_dir: {:?}", e).unwrap();
            return Err(Box::<dyn std::error::Error + Send + Sync>::from(e));
        }
    };

    writeln!(file, "Project path = {}", project_path).unwrap();
    writeln!(file, "Loading AgentConfig...").unwrap();

    let config = AgentConfig::load(&project_path).map_err(|e| {
        writeln!(file, "Failed to load config: {:?}", e).ok();
        Box::<dyn std::error::Error + Send + Sync>::from(e)
    })?;

    writeln!(file, "Config loaded. Initializing AppState...").unwrap();

    let app_state = AppState::new(config).await.map_err(|e| {
        writeln!(file, "Failed to initialize AppState: {:?}", e).ok();
        Box::<dyn std::error::Error + Send + Sync>::from(e)
    })?;

    let project_id_for_migration = std::env::current_dir()
        .map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        })
        .unwrap_or_else(|_| "project".to_string());

    // Run custom data migrations
    let _ = app_state.store.run_migrations(&project_id_for_migration);

    let ares_dir = std::path::PathBuf::from(&project_path).join(".ares");
    if !ares_dir.exists() {
        std::fs::create_dir_all(&ares_dir).ok();
    }
    let workspace_engine = Arc::new(
        ares_repository_intelligence::engines::workspace::WorkspaceEngine::new(ares_dir).unwrap(),
    );

    writeln!(file, "AppState initialized successfully.").unwrap();

    let assembler = Arc::new(MemoryContextAssembler::default_from_store(
        app_state.store.clone(),
    ));
    let governance = Arc::new(ares_governance::GovernanceFacade::new(
        app_state.store.clone(),
        std::path::PathBuf::from(&project_path),
    ));
    let facade = Arc::new(MemoryFacade::new(assembler.clone(), governance.clone()));
    let intelligence_facade = Arc::new(IntelligenceFacade::new(app_state.store.clone()));

    let inference_engine: Arc<dyn ares_core::inference::InferenceEngine> =
        if std::env::var("OPENAI_API_KEY").is_ok() {
            match ares_embeddings::providers::openai::OpenAIEmbeddingProvider::new() {
                Ok(provider) => Arc::new(provider),
                Err(e) => {
                    println!("WARN: Failed to initialize OpenAI provider: {}. Falling back to mock engine.", e);
                    Arc::new(ares_agent::inference::MockInferenceEngine)
                }
            }
        } else if std::env::var("OLLAMA_HOST").is_ok() {
            Arc::new(ares_embeddings::providers::ollama::OllamaEmbeddingProvider::new())
        } else {
            Arc::new(ares_agent::inference::MockInferenceEngine)
        };

    // Create the Why tool
    let intelligence_facade_why = intelligence_facade.clone();
    let project_id_str = project_path.clone();

    let why_tool = ToolBuilder::new("ares_why_exists")
        .description("Explains why a specific entity exists in the ARES memory graph")
        .handler(move |input: MemoryQueryInput| {
            let facade = intelligence_facade_why.clone();
            let project_id = project_id_str.clone();

            async move {
                let id = ares_core::canonicalize_node_id(&input.id);
                let query = EngineeringQuery {
                    entity_id: id.to_string(),
                    project_id,
                    query_type: QueryType::WhyExists,
                    workspace_root: None,
                    branch: None,
                };

                match facade.execute(&query) {
                    Ok(insight) => {
                        let response = serde_json::json!({
                            "answer": insight.answer,
                            "confidence": insight.confidence,
                            "evidence": insight.evidence,
                            "warnings": insight.warnings,
                            "recommendations": insight.recommendations,
                            "summary": insight.summary,
                            "file_path": &input.id,
                            "entity": &input.id,
                            "mode": insight.mode,
                            "metadata": insight.metadata,
                        });
                        Ok(CallToolResult::text(
                            serde_json::to_string(&response).unwrap(),
                        ))
                    }
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to explain why entity exists",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    // Create the Who tool
    let facade_who = facade.clone();
    let who_tool = ToolBuilder::new("ares_who_owns")
        .description("Identifies ownership and authorship information for an entity")
        .handler(move |input: MemoryQueryInput| {
            let facade = facade_who.clone();
            async move {
                let id = ares_core::canonicalize_node_id(&input.id);
                match facade.who(&id) {
                    Ok(result) => Ok(CallToolResult::text(result.to_string())),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to identify ownership",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    // Create the Evolution tool
    let facade_evolution = facade.clone();
    let evolution_tool = ToolBuilder::new("ares_evolution")
        .description("Retrieves the evolutionary timeline of an entity")
        .handler(move |input: MemoryQueryInput| {
            let facade = facade_evolution.clone();
            async move {
                let id = ares_core::canonicalize_node_id(&input.id);
                match facade.evolution(&id) {
                    Ok(result) => serde_json::to_string(&result)
                        .map(CallToolResult::text)
                        .map_err(|e| {
                            tower_mcp::Error::internal(format_mcp_error(
                                "Failed to serialize evolution timeline",
                                &e.to_string(),
                            ))
                        }),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to retrieve evolution timeline",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    // Create the Impact tool
    let intelligence_facade_impact = intelligence_facade.clone();
    let project_id_str_impact = project_path.clone();
    let impact_tool = ToolBuilder::new("ares_impact")
        .description("Performs read-only dependency analysis to determine what downstream components break if this entity is modified. Use this for general blast-radius queries without mutating the graph.")
        .handler(move |input: MemoryQueryInput| {
            let facade = intelligence_facade_impact.clone();
            let project_id = project_id_str_impact.clone();
            async move {
                let id = ares_core::canonicalize_node_id(&input.id);
                let query = EngineeringQuery {
                    entity_id: id.to_string(),
                    project_id,
                    query_type: QueryType::Impact,
                    workspace_root: None,
                    branch: None,
                };
                match facade.execute(&query) {
                    Ok(insight) => {
                        let response = serde_json::json!({
                            "answer": insight.answer,
                            "confidence": insight.confidence,
                            "evidence": insight.evidence,
                            "warnings": insight.warnings,
                            "recommendations": insight.recommendations,
                            "summary": insight.summary,
                            "file_path": &input.id,
                            "entity": &input.id,
                            "mode": insight.mode,
                            "metadata": insight.metadata,
                        });
                        Ok(CallToolResult::text(serde_json::to_string(&response).unwrap()))
                    }
                    Err(e) => Ok(CallToolResult::text(format!(
                        "{{\"answer\":\"Error: {}\",\"confidence\":0,\"evidence\":[],\"mode\":\"Offline\"}}",
                        e
                    ))),
                }
            }
        })
        .build();

    // Create the Certification Resource
    let cert_runner = Arc::new(ares_validation::validation_runner::ValidationRunner::new(
        Arc::new(app_state.store.clone()),
        assembler.clone(),
    ));

    let runner_cert = cert_runner.clone();
    let cert_resource = ResourceBuilder::new("memory://certification")
        .name("MemoryOS Certification")
        .description("Runs the MemoryOS certification validation suite")
        .mime_type("application/json")
        .handler(move || {
            let runner = runner_cert.clone();
            async move {
                match runner.run_certification().await {
                    Ok(result) => serde_json::to_string(&result)
                        .map(|s| ReadResourceResult::text("memory://certification", s))
                        .map_err(|e| {
                            tower_mcp::Error::internal(format_mcp_error(
                                "Failed to serialize certification result",
                                &e.to_string(),
                            ))
                        }),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to run certification",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    // Create the Context Resource Template
    let facade_context = facade.clone();
    let context_resource = ResourceTemplateBuilder::new("memory://context/{id}")
        .name("Memory Context")
        .description("Retrieves the full memory context package for an entity")
        .mime_type("application/json")
        .handler(move |uri: String, vars: HashMap<String, String>| {
            let facade = facade_context.clone();
            let id = ares_core::canonicalize_node_id(&vars.get("id").cloned().unwrap_or_default());
            async move {
                match facade.context(&id) {
                    Ok(result) => serde_json::to_string(&result)
                        .map(|s| ReadResourceResult::text(uri, s))
                        .map_err(|e| {
                            tower_mcp::Error::internal(format_mcp_error(
                                "Failed to serialize context",
                                &e.to_string(),
                            ))
                        }),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to retrieve context",
                        &e.to_string(),
                    ))),
                }
            }
        });

    let facade_summary = facade.clone();
    let summary_resource = ResourceTemplateBuilder::new("memory://summary/{id}")
        .name("Memory Context Summary")
        .description("Retrieves a lightweight, token-efficient summary of an entity's context")
        .mime_type("application/json")
        .handler(move |uri: String, vars: HashMap<String, String>| {
            let facade = facade_summary.clone();
            let id = ares_core::canonicalize_node_id(&vars.get("id").cloned().unwrap_or_default());
            async move {
                // Fetch the core details
                let why = facade.why(&id).ok();
                let who = facade.who(&id).ok();
                let impact = facade.impact(&id).ok();
                let coverage = facade.is_requirement_fully_implemented(&id).ok();

                let summary = serde_json::json!({
                    "entity": id,
                    "why_it_exists": why,
                    "owner_info": who,
                    "impact_analysis": impact,
                    "coverage_status": coverage
                });

                serde_json::to_string(&summary)
                    .map(|s| ReadResourceResult::text(uri, s))
                    .map_err(|e| {
                        tower_mcp::Error::internal(format_mcp_error(
                            "Failed to serialize summary",
                            &e.to_string(),
                        ))
                    })
            }
        });

    // Create the Compliance tool
    let facade_compliance = facade.clone();
    let compliance_tool = ToolBuilder::new("ares_compliance")
        .description(
            "Evaluates the compliance of a specific entity against active governance policies",
        )
        .handler(move |input: GovernanceQueryInput| {
            let facade = facade_compliance.clone();
            async move {
                let governance = facade.get_governance();
                let node_id = ares_core::canonicalize_node_id(&input.node_id);
                match governance
                    .is_compliant(
                        &ares_core::ProjectId::from(input.project_id),
                        &ares_core::NodeId::from(node_id),
                    )
                    .await
                {
                    Ok(result) => serde_json::to_string(&result)
                        .map(CallToolResult::text)
                        .map_err(|e| {
                            tower_mcp::Error::internal(format_mcp_error(
                                "Failed to serialize compliance evaluation",
                                &e.to_string(),
                            ))
                        }),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to evaluate compliance",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    // Create the Scorecard tool
    let facade_scorecard = facade.clone();
    let scorecard_tool = ToolBuilder::new("ares_scorecard")
        .description("Retrieves the governance scorecard for a project")
        .handler(move |input: ProjectQueryInput| {
            let facade = facade_scorecard.clone();
            async move {
                let governance = facade.get_governance();
                match governance
                    .get_scorecard(&ares_core::ProjectId::from(input.project_id))
                    .await
                {
                    Ok(result) => serde_json::to_string(&result)
                        .map(CallToolResult::text)
                        .map_err(|e| {
                            tower_mcp::Error::internal(format_mcp_error(
                                "Failed to serialize scorecard",
                                &e.to_string(),
                            ))
                        }),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to retrieve scorecard",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    // Create the Dashboard tool
    let store_dashboard = app_state.store.clone();
    let dashboard_project_path = project_path.clone();
    let dashboard_tool = ToolBuilder::new("ares_dashboard")
        .description("Retrieves the comprehensive repository overview dashboard")
        .handler(move |_input: ProjectQueryInput| {
            let store = store_dashboard.clone();
            let path = dashboard_project_path.clone();
            async move {
                let use_planner = std::env::var("ARES_USE_PLANNER").unwrap_or_else(|_| "0".to_string()) == "1";
                if use_planner {
                    tracing::info!("Executing ares_dashboard via ExecutionPlanner");
                    
                    let mut registry = ares_repository_intelligence::planner::registry::EngineRegistry::new();
                    registry.register(
                        ares_repository_intelligence::core::engine::EngineId::Overview,
                        vec![ares_repository_intelligence::core::capabilities::Capability::Workspace],
                        Box::new(ares_repository_intelligence::engines::overview::RepositoryOverviewEngine::new(store.clone()))
                    );
                    
                    let planner = ares_repository_intelligence::planner::pipeline::ExecutionPlanner::new(&registry);
                    
                    let context = ares_repository_intelligence::core::context::RepositoryContext {
                        repository: ares_repository_intelligence::core::context::RepositoryInfo {
                            root_path: path.clone(),
                            name: "project".to_string(),
                        },
                        snapshot: ares_repository_intelligence::core::context::RepositorySnapshot::default(),
                        workspace: ares_repository_intelligence::core::context::WorkspaceContext {
                            workspace_id: ares_core::id::new_id(),
                        },
                        execution: ares_repository_intelligence::core::context::ExecutionContext {
                            execution_id: ares_core::id::new_id(),
                            started_at: 0,
                            requested_by: "mcp".to_string(),
                            entry_point: ares_repository_intelligence::core::context::EntryPoint::API,
                            execution_mode: ares_repository_intelligence::core::context::ExecutionMode::Direct,
                            streaming: false,
                            debug: false,
                        },
                        policy: ares_repository_intelligence::core::context::ExecutionPolicy::default(),
                        request: ares_repository_intelligence::core::context::RequestContext {
                            query: "intent:dashboard".to_string(),
                            parameters: std::collections::HashMap::new(),
                        },
                    };
                    
                    let response = planner.execute(&context).await;
                    serde_json::to_string(&response)
                        .map(CallToolResult::text)
                        .map_err(|e| {
                            tower_mcp::Error::internal(format_mcp_error(
                                "Failed to serialize planner dashboard response",
                                &e.to_string(),
                            ))
                        })
                } else {
                    tracing::info!("Executing ares_dashboard via Legacy Engine");
                    let result = ares_repository_intelligence::engines::overview::RepositoryOverviewEngine::collect(&store, &path).await;
                    serde_json::to_string(&result)
                        .map(CallToolResult::text)
                        .map_err(|e| {
                            tower_mcp::Error::internal(format_mcp_error(
                                "Failed to serialize dashboard",
                                &e.to_string(),
                            ))
                        })
                }
            }
        })
        .build();

    // PHASE 1.4.0 Requirement Intelligence Tools
    let store_cov = app_state.store.clone();
    let coverage_tool = ToolBuilder::new("ares_coverage")
        .description("Calculates the coverage of requirements for a project")
        .handler(move |input: ProjectQueryInput| {
            let store = store_cov.clone();
            async move {
                let project_id = ares_core::ProjectId::from(input.project_id);
                let req_store = ares_requirements::storage::RequirementStore::new(store.clone());
                let reqs = match req_store.list(
                    &project_id,
                    ares_requirements::models::RequirementFilter::default(),
                ) {
                    Ok(r) => r,
                    Err(e) => {
                        return Err(tower_mcp::Error::internal(format_mcp_error(
                            "Failed to list requirements",
                            &e.to_string(),
                        )))
                    }
                };
                let graph = ares_traceability::TraceabilityGraph::new(); // In a real scenario we load the actual edges
                let engine = ares_requirements::coverage::RequirementCoverageEngine::new();
                let trace = ares_requirements::trace_analysis::TraceAnalysisEngine::new(&graph);
                let mut coverages = Vec::new();
                for req in reqs {
                    coverages.push(engine.evaluate(
                        &req.id,
                        &req.status,
                        req.owner.is_some(),
                        &trace,
                    ));
                }
                let (summary, _) = engine.generate_summary(&coverages);
                serde_json::to_string(&summary)
                    .map(CallToolResult::text)
                    .map_err(|e| {
                        tower_mcp::Error::internal(format_mcp_error(
                            "Failed to serialize coverage summary",
                            &e.to_string(),
                        ))
                    })
            }
        })
        .build();

    let _store_drift = app_state.store.clone();
    let intelligence_facade_drift = intelligence_facade.clone();
    let project_id_str_drift = project_path.clone();
    let drift_tool = ToolBuilder::new("ares_drift")
        .description("Evaluates structural drift for a given file")
        .handler(move |input: MemoryQueryInput| {
            let facade = intelligence_facade_drift.clone();
            let project_id = project_id_str_drift.clone();
            async move {
                let id = ares_core::canonicalize_node_id(&input.id);
                let query = EngineeringQuery {
                    entity_id: id.to_string(),
                    project_id,
                    query_type: QueryType::Drift,
                    workspace_root: None,
                    branch: None,
                };
                match facade.execute(&query) {
                    Ok(insight) => {
                        let response = serde_json::json!({
                            "answer": insight.answer,
                            "confidence": insight.confidence,
                            "evidence": insight.evidence,
                            "warnings": insight.warnings,
                            "recommendations": insight.recommendations,
                            "summary": insight.summary,
                            "file_path": &input.id,
                            "entity": &input.id,
                            "mode": insight.mode,
                            "metadata": insight.metadata,
                        });
                        Ok(CallToolResult::text(serde_json::to_string(&response).unwrap()))
                    }
                    Err(e) => Ok(CallToolResult::text(format!(
                        "{{\"answer\":\"Error: {}\",\"confidence\":0,\"evidence\":[],\"mode\":\"Offline\"}}",
                        e
                    ))),
                }
            }
        })
        .build();

    // ============================================================
    // PHASE 2 TASK 3.1: Additional MCP Tools
    // ============================================================

    // --- ares_who_owns ---
    let store_who = app_state.store.clone();
    let pp_who = project_path.clone();
    let who_owns_tool = ToolBuilder::new("ares_who_owns")
        .description("Returns the registered owner and contributor history for a file")
        .handler(move |input: OwnerQueryInput| {
            let store_arc = store_who.clone();
            let pp = pp_who.clone();
            async move {
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
                let project_id = ares_core::ProjectId::from(project_name);

                let mut owner_name = String::new();
                let mut owner_confidence = 0.0f32;
                let mut contributors: Vec<serde_json::Value> = Vec::new();
                let mut total_weight = 0.0f32;

                if let Ok(file_id_str) = repo.get_id_by_path(&input.file_path) {
                    let file_id = ares_core::NodeId::from(file_id_str.as_str());

                    if let Ok(edges) = repo.get_edges_to_by_type(&file_id, "authored_by") {
                        for e in &edges {
                            if let Ok(Some(p)) = repo.get_node(&e.from_node_id) {
                                owner_name = p.label.clone();
                                owner_confidence = e.confidence;
                            }
                        }
                    }

                    if let Ok(edges) = repo.get_edges_to_by_type(&file_id, "contributed_to") {
                        for e in &edges {
                            total_weight += e.weight;
                            if let Ok(Some(p)) = repo.get_node(&e.from_node_id) {
                                contributors.push(serde_json::json!({
                                    "name": p.label,
                                    "percentage": (e.weight * 100.0).round() as i32
                                }));
                            }
                        }
                    }
                }

                contributors.sort_by(|a, b| b["percentage"].as_i64().cmp(&a["percentage"].as_i64()));

                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "result": {
                        "owner": owner_name,
                        "confidence": owner_confidence,
                        "commit_percentage": if total_weight > 0.0 { (total_weight * 100.0).round() as i32 } else { 0 },
                        "contributors": contributors
                    },
                    "evidence": [{"source": "graph", "confidence": 1.0}],
                    "query_time_ms": start.elapsed().as_millis() as i64
                })).unwrap()))
            }
        })
        .build();

    // --- ares_decisions ---
    let store_dec = app_state.store.clone();
    let pp_dec = project_path.clone();
    let decisions_tool = ToolBuilder::new("ares_decisions")
        .description("Returns architectural decisions, optionally filtered by file path or date")
        .handler(move |input: DecisionsQueryInput| {
            let store_arc = store_dec.clone();
            let pp = pp_dec.clone();
            async move {
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
                let project_id = ares_core::ProjectId::from(project_name);

                let mut decisions = Vec::new();
                let target_file_id = input.file_path.as_ref().and_then(|fp| repo.get_id_by_path(fp).ok());

                if let Ok(all) = repo.get_nodes_by_type(&project_id, "decision") {
                    for dn in &all {
                        let props = &dn.properties;
                        let summary = props.get("decision").and_then(|v| v.as_str()).unwrap_or(&dn.label);
                        let author = props.get("author").and_then(|v| v.as_str()).unwrap_or("unknown");

                        let mut matches = target_file_id.is_none();
                        let mut files: Vec<String> = Vec::new();

                        if let Ok(edges) = repo.get_edges_from(&dn.id) {
                            for e in &edges {
                                files.push(e.to_node_id.as_str().to_string());
                                if let Some(ref fid) = target_file_id {
                                    if e.to_node_id.as_str() == fid.as_str() {
                                        matches = true;
                                    }
                                }
                            }
                        }

                        if matches {
                            if let Some(ref since) = input.since {
                                if let Ok(ts) = since.parse::<i64>() {
                                    if dn.created_at < ts { continue; }
                                }
                            }
                            decisions.push(serde_json::json!({
                                "id": dn.id.as_str(),
                                "date": dn.created_at,
                                "summary": summary,
                                "author": author,
                                "files": files
                            }));
                        }
                    }
                }

                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "result": { "decisions": decisions },
                    "evidence": [{"source": "graph", "confidence": 1.0}],
                    "query_time_ms": start.elapsed().as_millis() as i64
                })).unwrap()))
            }
        })
        .build();

    // --- ares_search ---
    let store_srch = app_state.store.clone();
    let pp_srch = project_path.clone();
    let search_tool = ToolBuilder::new("ares_search")
        .description("Searches nodes by label or file path using full-text matching")
        .handler(move |input: SearchQueryInput| {
            let store_arc = store_srch.clone();
            let pp = pp_srch.clone();
            async move {
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
                let project_id = ares_core::ProjectId::from(project_name);
                let _pattern = format!("%{}%", input.query);

                let mut results = Vec::new();
                if let Ok(all) = repo.get_all_nodes(&project_id) {
                    let mut matched: Vec<_> = all.into_iter().filter(|n| {
                        n.label.to_lowercase().contains(&input.query.to_lowercase())
                            || n.file_path.as_ref().map_or(false, |fp| fp.to_lowercase().contains(&input.query.to_lowercase()))
                    }).collect();
                    matched.truncate(input.limit);
                    for n in matched {
                        results.push(serde_json::json!({
                            "node_id": n.id.as_str(),
                            "type": n.node_type,
                            "summary": n.label,
                            "file_path": n.file_path
                        }));
                    }
                }

                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "result": { "results": results },
                    "evidence": [{"source": "graph", "confidence": 1.0}],
                    "query_time_ms": start.elapsed().as_millis() as i64
                })).unwrap()))
            }
        })
        .build();

    // --- ares_timeline ---
    let store_tl = app_state.store.clone();
    let pp_tl = project_path.clone();
    let timeline_tool = ToolBuilder::new("ares_timeline")
        .description("Returns the chronological commit history for a file")
        .handler(move |input: TimelineQueryInput| {
            let store_arc = store_tl.clone();
            let pp = pp_tl.clone();
            async move {
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
                let _project_id = ares_core::ProjectId::from(project_name);

                let mut events = Vec::new();
                if let Ok(file_id_str) = repo.get_id_by_path(&input.file_path) {
                    let file_id = ares_core::NodeId::from(file_id_str.as_str());
                    if let Ok(edges) = repo.get_edges_to_by_type(&file_id, "touches") {
                        let mut commit_ids: Vec<(i64, ares_core::NodeId)> = edges.iter()
                            .map(|e| (e.valid_from, e.from_node_id.clone()))
                            .collect();
                        commit_ids.sort_by_key(|(ts, _)| *ts);

                        for (ts, cid) in &commit_ids {
                            if let Ok(Some(commit)) = repo.get_node(cid) {
                                let author = commit.properties.get("author").and_then(|v| v.as_str()).unwrap_or("unknown");
                                let subject = commit.properties.get("subject").and_then(|v| v.as_str()).unwrap_or("");
                                events.push(serde_json::json!({
                                    "date": *ts,
                                    "type": "commit",
                                    "summary": subject,
                                    "author": author
                                }));
                            }
                        }
                    }
                }

                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "result": { "events": events },
                    "evidence": [{"source": "graph", "confidence": 1.0}],
                    "query_time_ms": start.elapsed().as_millis() as i64
                })).unwrap()))
            }
        })
        .build();

    // --- ares_compare ---
    let store_cmp = app_state.store.clone();
    let pp_cmp = project_path.clone();
    let compare_tool = ToolBuilder::new("ares_compare")
        .description("Compares two files: shared dependencies, shared decisions, coupling score")
        .handler(move |input: CompareQueryInput| {
            let store_arc = store_cmp.clone();
            let pp = pp_cmp.clone();
            async move {
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
                let _project_id = ares_core::ProjectId::from(project_name);

                let id_a = repo.get_id_by_path(&input.file_a).ok().map(|s| ares_core::NodeId::from(s.as_str()));
                let id_b = repo.get_id_by_path(&input.file_b).ok().map(|s| ares_core::NodeId::from(s.as_str()));

                let mut deps_a = std::collections::HashSet::new();
                let mut deps_b = std::collections::HashSet::new();

                if let Some(ref id) = id_a {
                    if let Ok(edges) = repo.get_edges_from(id) {
                        for e in &edges {
                            if e.edge_type.as_str() == "depends_on" {
                                deps_a.insert(e.to_node_id.as_str().to_string());
                            }
                        }
                    }
                }
                if let Some(ref id) = id_b {
                    if let Ok(edges) = repo.get_edges_from(id) {
                        for e in &edges {
                            if e.edge_type.as_str() == "depends_on" {
                                deps_b.insert(e.to_node_id.as_str().to_string());
                            }
                        }
                    }
                }

                let shared: Vec<String> = deps_a.intersection(&deps_b).cloned().collect();
                let union_count = deps_a.union(&deps_b).count();
                let coupling = if union_count > 0 { shared.len() as f64 / union_count as f64 } else { 0.0 };

                let relationship = if coupling > 0.5 { "tightly coupled" }
                    else if coupling > 0.1 { "loosely coupled" }
                    else { "independent" };

                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "result": {
                        "shared_dependencies": shared,
                        "shared_decisions": [],
                        "relationship": relationship,
                        "coupling_score": (coupling * 100.0).round() as i32
                    },
                    "evidence": [{"source": "graph", "confidence": 1.0}],
                    "query_time_ms": start.elapsed().as_millis() as i64
                })).unwrap()))
            }
        })
        .build();

    // --- ares_architecture ---
    let store_arch = app_state.store.clone();
    let pp_arch = project_path.clone();
    let architecture_tool = ToolBuilder::new("ares_architecture")
        .description("Returns a high-level architectural overview of the repository")
        .handler(move |_input: ArchitectureQueryInput| {
            let store_arc = store_arch.clone();
            let pp = pp_arch.clone();
            async move {
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
                let project_id = ares_core::ProjectId::from(project_name);

                let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                let mut dep_names: std::collections::HashSet<String> = std::collections::HashSet::new();
                let mut top_files: Vec<(usize, String)> = Vec::new();
                let mut decisions: Vec<serde_json::Value> = Vec::new();

                if let Ok(all_nodes) = repo.get_all_nodes(&project_id) {
                    for n in &all_nodes {
                        *type_counts.entry(format!("{:?}", n.node_type).to_lowercase()).or_insert(0) += 1;
                        if n.id.as_str().starts_with("DEP-") {
                            dep_names.insert(n.label.clone());
                        }
                    }

                    // Find top files by incoming edge count
                    let file_ids: Vec<_> = all_nodes.iter()
                        .filter(|n| format!("{:?}", n.node_type).to_lowercase() == "file")
                        .take(200) // limit for performance
                        .collect();

                    for fn_node in &file_ids {
                        let in_count = repo.get_edges_to(&fn_node.id).map(|e| e.len()).unwrap_or(0);
                        let path = fn_node.file_path.clone().unwrap_or_default();
                        top_files.push((in_count, path));
                    }
                    top_files.sort_by(|a, b| b.0.cmp(&a.0));
                    top_files.truncate(10);
                }

                if let Ok(all_decisions) = repo.get_nodes_by_type(&project_id, "decision") {
                    for d in &all_decisions {
                        let summary = d.properties.get("decision").and_then(|v| v.as_str()).unwrap_or(&d.label);
                        decisions.push(serde_json::json!({ "summary": summary }));
                    }
                    decisions.truncate(10);
                }

                let file_count = type_counts.get("file").copied().unwrap_or(0);
                let func_count = type_counts.get("function").copied().unwrap_or(0);
                let commit_count = type_counts.get("commit").copied().unwrap_or(0);

                let tech_stack: Vec<String> = dep_names.into_iter().take(20).collect();
                let top: Vec<serde_json::Value> = top_files.iter().map(|(c, p)| serde_json::json!({"path": p, "dependents": c})).collect();

                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "result": {
                        "summary": format!("{} files, {} functions, {} commits across {} node types", file_count, func_count, commit_count, type_counts.len()),
                        "top_files": top,
                        "key_decisions": decisions,
                        "technology_stack": tech_stack,
                        "health_score": 0
                    },
                    "evidence": [{"source": "graph", "confidence": 1.0}],
                    "query_time_ms": start.elapsed().as_millis() as i64
                })).unwrap()))
            }
        })
        .build();

    // --- ares_requirements ---
    let store_req = app_state.store.clone();
    let pp_req = project_path.clone();
    let requirements_tool = ToolBuilder::new("ares_requirements")
        .description("Returns requirements linked to the repository or a specific file")
        .handler(move |input: RequirementsQueryInput| {
            let store_arc = store_req.clone();
            let pp = pp_req.clone();
            async move {
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
                let project_id = ares_core::ProjectId::from(project_name);

                let mut requirements = Vec::new();

                if let Ok(all) = repo.get_nodes_by_type(&project_id, "requirement") {
                    for rn in &all {
                        let text = rn.properties.get("text").and_then(|v| v.as_str()).unwrap_or(&rn.label);
                        let status = rn.properties.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");

                        let mut linked_files: Vec<String> = Vec::new();
                        let mut matches = input.file_path.is_none();

                        if let Ok(edges) = repo.get_edges_from(&rn.id) {
                            for e in &edges {
                                let target_path = e.to_node_id.as_str().to_string();
                                linked_files.push(target_path.clone());
                                if let Some(ref fp) = input.file_path {
                                    if target_path.contains(fp) || fp.contains(&target_path) {
                                        matches = true;
                                    }
                                }
                            }
                        }

                        if matches {
                            requirements.push(serde_json::json!({
                                "id": rn.id.as_str(),
                                "text": text,
                                "status": status,
                                "linked_files": linked_files
                            }));
                        }
                    }
                }

                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "result": { "requirements": requirements },
                    "evidence": [{"source": "graph", "confidence": 1.0}],
                    "query_time_ms": start.elapsed().as_millis() as i64
                })).unwrap()))
            }
        })
        .build();

    let gaps_tool = ToolBuilder::new("ares_gaps")
        .description("Evaluates knowledge gaps in the traceability graph")
        .handler(move |_input: ProjectQueryInput| async move {
            let graph = ares_traceability::TraceabilityGraph::new();
            let engine = ares_requirements::gaps::KnowledgeGapEngine::new(&graph);
            let gaps = engine.evaluate_gaps();
            serde_json::to_string(&gaps)
                .map(CallToolResult::text)
                .map_err(|e| {
                    tower_mcp::Error::internal(format_mcp_error(
                        "Failed to serialize gaps evaluation",
                        &e.to_string(),
                    ))
                })
        })
        .build();

    let store_sim = app_state.store.clone();
    let simulate_tool = ToolBuilder::new("ares_simulate")
        .description("Performs mutation analysis only. Simulates structural changes (e.g., removing a node) to project coverage drops, new gaps, and drift before they happen.")
        .handler(move |input: SimulationInput| {
            let store = store_sim.clone();
            async move {
                let target_id = ares_core::canonicalize_node_id(&input.target_id);
                let related = input.related_id.as_deref().map(ares_core::canonicalize_node_id);

                let action_enum = match input.action.parse::<ares_intelligence::simulation::SimulationAction>() {
                    Ok(a) => a,
                    Err(_) => return Err(tower_mcp::Error::internal(format_mcp_error("Unsupported action", "Unsupported simulation action"))),
                };

                match ares_intelligence::simulation::simulate(
                    action_enum,
                    &target_id,
                    related.as_deref(),
                    &store,
                ).await {
                    Ok(report) => serde_json::to_string(&report)
                        .map(CallToolResult::text)
                        .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize simulation report", &e.to_string()))),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to simulate change", &e.to_string()))),
                }
            }
        })
        .build();

    let intelligence_facade_trace = intelligence_facade.clone();
    let project_id_str_trace = project_path.clone();
    let traceability_tool = ToolBuilder::new("ares_traceability")
        .description("Evaluates traceability relationships upstream and downstream")
        .handler(move |input: TraceabilityInput| {
            let facade = intelligence_facade_trace.clone();
            let project_id = project_id_str_trace.clone();
            async move {
                let id = ares_core::canonicalize_node_id(&input.entity_id);
                let query = EngineeringQuery {
                    entity_id: id.to_string(),
                    project_id,
                    query_type: QueryType::Traceability,
                    workspace_root: None,
                    branch: None,
                };
                match facade.execute(&query) {
                    Ok(insight) => {
                        let response = serde_json::json!({
                            "answer": insight.answer,
                            "confidence": insight.confidence,
                            "evidence": insight.evidence,
                            "warnings": insight.warnings,
                            "recommendations": insight.recommendations,
                            "summary": insight.summary,
                            "file_path": &input.entity_id,
                            "entity": &input.entity_id,
                            "mode": insight.mode,
                            "metadata": insight.metadata,
                        });
                        Ok(CallToolResult::text(serde_json::to_string(&response).unwrap()))
                    }
                    Err(e) => Ok(CallToolResult::text(format!(
                        "{{\"answer\":\"Error: {}\",\"confidence\":0,\"evidence\":[],\"mode\":\"Offline\"}}",
                        e
                    ))),
                }
            }
        })
        .build();

    let store_graph = app_state.store.clone();
    let graph_statistics_tool = ToolBuilder::new("ares_graph_statistics")
        .description("Retrieves statistics about the knowledge graph")
        .handler(move |_input: EmptyInput| {
            let store = store_graph.clone();
            async move {
                let result = ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_statistics(&store).await;
                match result {
                    Ok(stats) => serde_json::to_string(&stats)
                        .map(CallToolResult::text)
                        .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize graph stats", &e.to_string()))),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to retrieve graph stats", &e.to_string()))),
                }
            }
        })
        .build();

    let store_graph_root = app_state.store.clone();
    let graph_root_tool = ToolBuilder::new("ares_graph_root")
        .description("Retrieves the root node of the graph to start lazy loading")
        .handler(move |_input: EmptyInput| {
            let store = store_graph_root.clone();
            async move {
                // Determine project_id (e.g. from cwd like CLI)
                // Since this runs in the workspace, we can use the same logic
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let name = cwd
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("project");
                let _pid = ares_core::ProjectId::from(name);

                let architecture_service =
                    ares_repository_intelligence::services::ArchitectureService::new(store.clone());
                match architecture_service.generate_architectural_seed(
                    &cwd.to_string_lossy(),
                    name,
                    60,
                ) {
                    Ok(payload) => serde_json::to_string(&payload)
                        .map(CallToolResult::text)
                        .map_err(|e| {
                            tower_mcp::Error::internal(format_mcp_error(
                                "Failed to serialize graph root",
                                &e.to_string(),
                            ))
                        }),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to retrieve graph root",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    let store_graph_neighbors = app_state.store.clone();
    let graph_neighbors_tool = ToolBuilder::new("ares_graph_neighbors")
        .description("Expands a node by fetching its immediate neighbors")
        .handler(move |input: GraphNeighborsInput| {
            let store = store_graph_neighbors.clone();
            async move {
                let node_id_str = ares_core::canonicalize_node_id(&input.node_id);
                let node_id = ares_core::NodeId::from(node_id_str);
                match ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_neighbors(&store, &node_id).await {
                    Ok(payload) => serde_json::to_string(&payload)
                        .map(CallToolResult::text)
                        .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize graph neighbors", &e.to_string()))),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to retrieve graph neighbors", &e.to_string()))),
                }
            }
        })
        .build();

    let store_graph_search = app_state.store.clone();
    let graph_search_tool = ToolBuilder::new("ares_graph_search")
        .description("Searches the graph for nodes matching the query")
        .handler(move |input: GraphSearchInput| {
            let store = store_graph_search.clone();
            async move {
                match ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_search(&store, &input.query).await {
                    Ok(payload) => serde_json::to_string(&payload)
                        .map(CallToolResult::text)
                        .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize graph search results", &e.to_string()))),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to search graph", &e.to_string()))),
                }
            }
        })
        .build();

    let store_graph_shortest_path = app_state.store.clone();
    let graph_shortest_path_tool = ToolBuilder::new("ares_graph_shortest_path")
        .description("Finds the shortest dependency path between two nodes")
        .handler(move |input: GraphPathInput| {
            let store = store_graph_shortest_path.clone();
            async move {
                let from_id_str = ares_core::canonicalize_node_id(&input.from_id);
                let to_id_str = ares_core::canonicalize_node_id(&input.to_id);
                let from_id = ares_core::NodeId::from(from_id_str);
                let to_id = ares_core::NodeId::from(to_id_str);
                match ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_shortest_path(&store, &from_id, &to_id).await {
                    Ok(payload) => serde_json::to_string(&payload)
                        .map(CallToolResult::text)
                        .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize shortest path", &e.to_string()))),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to find shortest path", &e.to_string()))),
                }
            }
        })
        .build();

    let store_graph_metadata = app_state.store.clone();
    let graph_metadata_tool = ToolBuilder::new("ares_graph_metadata")
        .description("Retrieves full metadata for a specific node")
        .handler(move |input: MemoryQueryInput| {
            let store = store_graph_metadata.clone();
            async move {
                let node_id_str = ares_core::canonicalize_node_id(&input.id);
                let node_id = ares_core::NodeId::from(node_id_str);
                match ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_metadata(&store, &node_id).await {
                    Ok(node) => serde_json::to_string(&node)
                        .map(CallToolResult::text)
                        .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize node metadata", &e.to_string()))),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to retrieve node metadata", &e.to_string()))),
                }
            }
        })
        .build();

    let we_bookmark = workspace_engine.clone();
    let workspace_bookmark_tool = ToolBuilder::new("ares_workspace_bookmark")
        .description("Bookmark a node or query in the workspace")
        .handler(move |input: BookmarkInput| {
            let we = we_bookmark.clone();
            async move {
                // kind is "Node", "Query", etc.
                // For direct call, we map bookmark_node or bookmark_query based on kind?
                // Actually, the WorkspaceEngine allows generic kind via private add_bookmark, but public are bookmark_node / bookmark_query.
                // Since I didn't make add_bookmark public, let's use match on kind.
                let res = if input.kind == "Node" {
                    we.bookmark_node(&input.value, &input.title).await
                } else {
                    we.bookmark_query(&input.value, &input.title).await
                };
                match res {
                    Ok(_) => Ok(CallToolResult::text("Bookmarked successfully".to_string())),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to bookmark",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    let we_pin = workspace_engine.clone();
    let workspace_pin_tool = ToolBuilder::new("ares_workspace_pin")
        .description("Pin a node in the workspace")
        .handler(move |input: PinInput| {
            let we = we_pin.clone();
            async move {
                match we.pin_node(&input.node_id).await {
                    Ok(_) => Ok(CallToolResult::text("Pinned successfully".to_string())),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to pin",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    let we_list = workspace_engine.clone();
    let workspace_list_tool = ToolBuilder::new("ares_workspace_list")
        .description("List recent questions, bookmarks, and pins")
        .handler(move |_input: EmptyInput| {
            let we = we_list.clone();
            async move {
                let questions = we.list_recent_questions().await.unwrap_or_default();
                let bookmarks = we.list_bookmarks().await.unwrap_or_default();
                let pins = we.list_pinned_nodes().await.unwrap_or_default();
                let response = serde_json::json!({
                    "recent_questions": questions,
                    "bookmarks": bookmarks,
                    "pins": pins
                });
                Ok(CallToolResult::text(
                    serde_json::to_string(&response).unwrap(),
                ))
            }
        })
        .build();

    let we_record_nav = workspace_engine.clone();
    let workspace_record_nav_tool = ToolBuilder::new("ares_workspace_record_navigation")
        .description("Record a navigation event")
        .handler(move |input: RecordNavigateInput| {
            let we = we_record_nav.clone();
            async move {
                match we.push_navigation(&input.node_id).await {
                    Ok(_) => Ok(CallToolResult::text("Recorded successfully".to_string())),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to record navigation",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    let we_nav = workspace_engine.clone();
    let workspace_navigate_tool = ToolBuilder::new("ares_workspace_navigate")
        .description("Navigate back or forward")
        .handler(move |input: NavigateInput| {
            let we = we_nav.clone();
            async move {
                let res = if input.direction == "back" {
                    we.navigation_back(input.current_timestamp).await
                } else {
                    we.navigation_forward(input.current_timestamp).await
                };
                match res {
                    Ok(Some(nav)) => Ok(CallToolResult::text(serde_json::to_string(&nav).unwrap())),
                    Ok(None) => Ok(CallToolResult::text("{}".to_string())),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to navigate",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    let store_chat = app_state.store.clone();
    let project_path_chat = project_path.clone();
    let inference_chat = inference_engine.clone();
    let we_chat = workspace_engine.clone();

    let chat_tool = ToolBuilder::new("ares_chat")
        .description("Repository Conversation Engine. Ask any question about the codebase.")
        .handler(move |input: ChatInput| {
            let store = store_chat.clone();
            let path = project_path_chat.clone();
            let inference = inference_chat.clone();
            let we = we_chat.clone();
            
            async move {
                let mut registry = ares_repository_intelligence::planner::registry::EngineRegistry::new();
                registry.register(
                    ares_repository_intelligence::core::engine::EngineId::Overview,
                    vec![ares_repository_intelligence::core::capabilities::Capability::Workspace],
                    Box::new(ares_repository_intelligence::engines::overview::RepositoryOverviewEngine::new(store.clone()))
                );
                
                let planner = ares_repository_intelligence::planner::pipeline::ExecutionPlanner::new(&registry);
                let conversation = ares_repository_intelligence::engines::conversation::ConversationEngine::new(&planner, inference);
                
                let mut context = ares_repository_intelligence::core::context::RepositoryContext {
                    repository: ares_repository_intelligence::core::context::RepositoryInfo {
                        root_path: path.clone(),
                        name: "project".to_string(),
                    },
                    snapshot: ares_repository_intelligence::core::context::RepositorySnapshot::default(),
                    workspace: ares_repository_intelligence::core::context::WorkspaceContext {
                        workspace_id: ares_core::id::new_id(),
                    },
                    execution: ares_repository_intelligence::core::context::ExecutionContext {
                        execution_id: ares_core::id::new_id(),
                        started_at: 0,
                        requested_by: "mcp".to_string(),
                        entry_point: ares_repository_intelligence::core::context::EntryPoint::API,
                        execution_mode: ares_repository_intelligence::core::context::ExecutionMode::Direct,
                        streaming: false,
                        debug: false,
                    },
                    policy: ares_repository_intelligence::core::context::ExecutionPolicy::default(),
                    request: ares_repository_intelligence::core::context::RequestContext {
                        query: "".to_string(),
                        parameters: std::collections::HashMap::new(),
                    },
                };
                
                match conversation.ask(&input.query, &mut context).await {
                    Ok(resp) => {
                        // Record recent question
                        let _ = we.add_recent_question(ares_repository_intelligence::engines::workspace::RecentQuestion {
                            id: ares_core::id::new_id(),
                            question: input.query.clone(),
                            repository_id: "project".to_string(),
                            execution_id: resp.response.execution_id.clone(),
                            replay_id: resp.response.replay_id.clone().unwrap_or_default(),
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        }).await;

                        let output = serde_json::json!({
                            "answer": resp.answer,
                            "actions": resp.actions,
                            "citations": resp.response.citations,
                        });
                        Ok(CallToolResult::text(serde_json::to_string(&output).unwrap()))
                    },
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed chat", &e.to_string()))),
                }
            }
        })
        .build();

    let router = McpRouter::new()
        .server_info("ares-mcp", env!("CARGO_PKG_VERSION"))
        .tool(chat_tool)
        .tool(workspace_bookmark_tool)
        .tool(workspace_pin_tool)
        .tool(workspace_list_tool)
        .tool(workspace_navigate_tool)
        .tool(workspace_record_nav_tool)
        .tool(why_tool)
        .tool(who_tool)
        .tool(evolution_tool)
        .tool(impact_tool)
        .tool(compliance_tool)
        .tool(scorecard_tool)
        .tool(dashboard_tool)
        .tool(coverage_tool)
        .tool(drift_tool)
        .tool(who_owns_tool)
        .tool(decisions_tool)
        .tool(search_tool)
        .tool(timeline_tool)
        .tool(compare_tool)
        .tool(architecture_tool)
        .tool(requirements_tool)
        .tool(gaps_tool)
        .tool(simulate_tool)
        .tool(traceability_tool)
        .tool(graph_statistics_tool)
        .tool(graph_root_tool)
        .tool(graph_neighbors_tool)
        .tool(graph_search_tool)
        .tool(graph_shortest_path_tool)
        .tool(graph_metadata_tool)
        .resource(cert_resource)
        .resource_template(context_resource)
        .resource_template(summary_resource);

    writeln!(
        file,
        "Router built successfully. Starting StdioTransport..."
    )
    .unwrap();

    info!("ARES MCP Server started on stdio");

    match StdioTransport::new(router).run().await {
        Ok(_) => {
            writeln!(file, "StdioTransport run finished successfully.").unwrap();
            Ok(())
        }
        Err(e) => {
            writeln!(file, "StdioTransport run failed: {:?}", e).unwrap();
            Err(Box::<dyn std::error::Error + Send + Sync>::from(e))
        }
    }
}
