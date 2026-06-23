use ares_agent::config::AgentConfig;
use ares_app::AppState;
use ares_memory_intelligence::assembler::MemoryContextAssembler;
use ares_memory_intelligence::facade::MemoryFacade;
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
#[allow(dead_code)]
struct SimulationInput {
    project_id: String,
    action: String,
    target_id: String,
    target_type: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Basic tracing setup for MCP (use stderr for logs so stdio stdout is free for JSON-RPC)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting ARES MCP Server");

    let project_path = std::env::current_dir()
        .expect("Cannot determine current directory")
        .to_string_lossy()
        .to_string();

    let config = AgentConfig::load(&project_path)?;
    let app_state = AppState::new(config).await?;

    let assembler = Arc::new(MemoryContextAssembler::default_from_store(
        app_state.store.clone(),
    ));
    let governance = Arc::new(ares_governance::GovernanceFacade::new(
        app_state.store.clone(),
        std::path::PathBuf::from(&project_path),
    ));
    let facade = Arc::new(MemoryFacade::new(assembler.clone(), governance.clone()));

    // Create the Why tool
    let facade_why = facade.clone();
    let why_tool = ToolBuilder::new("ares_why_exists")
        .description("Explains why a specific entity exists in the ARES memory graph")
        .handler(move |input: MemoryQueryInput| {
            let facade = facade_why.clone();
            async move {
                let id = ares_core::canonicalize_node_id(&input.id);
                match facade.why(&id) {
                    Ok(result) => Ok(CallToolResult::text(result.to_string())),
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
    let facade_impact = facade.clone();
    let impact_tool = ToolBuilder::new("ares_impact")
        .description("Performs read-only dependency analysis to determine what downstream components break if this entity is modified. Use this for general blast-radius queries without mutating the graph.")
        .handler(move |input: MemoryQueryInput| {
            let facade = facade_impact.clone();
            async move {
                let id = ares_core::canonicalize_node_id(&input.id);
                match facade.impact(&id) {
                    Ok(result) => serde_json::to_string(&result)
                        .map(CallToolResult::text)
                        .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize impact analysis", &e.to_string()))),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to perform impact analysis", &e.to_string()))),
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

    let store_drift = app_state.store.clone();
    let drift_tool = ToolBuilder::new("ares_drift")
        .description("Evaluates structural drift for requirements")
        .handler(move |_input: ProjectQueryInput| {
            let _store = store_drift.clone();
            async move {
                // Mapped to graph traversal
                serde_json::to_string(
                    "Drift calculation requires historic baseline. Not fully implemented.",
                )
                .map(CallToolResult::text)
                .map_err(|e| {
                    tower_mcp::Error::internal(format_mcp_error(
                        "Failed to serialize drift message",
                        &e.to_string(),
                    ))
                })
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
                let project_id = ares_core::ProjectId::from(input.project_id);
                let engine = ares_requirements::simulation::RequirementSimulationEngine::new(std::sync::Arc::new(store));
                let target_id = ares_core::canonicalize_node_id(&input.target_id);
                let change = if input.action == "remove" {
                    ares_requirements::simulation::ProposedChange::RemoveNode { id: target_id }
                } else {
                    return Err(tower_mcp::Error::internal(format_mcp_error("Unsupported action", "Only 'remove' is supported")));
                };
                let graph = ares_traceability::TraceabilityGraph::new();
                match engine.simulate_change(&project_id, &graph, change) {
                    Ok(report) => serde_json::to_string(&report)
                        .map(CallToolResult::text)
                        .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize simulation report", &e.to_string()))),
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to simulate change", &e.to_string()))),
                }
            }
        })
        .build();

    let router = McpRouter::new()
        .server_info("ares-mcp", env!("CARGO_PKG_VERSION"))
        .tool(why_tool)
        .tool(who_tool)
        .tool(evolution_tool)
        .tool(impact_tool)
        .tool(compliance_tool)
        .tool(scorecard_tool)
        .tool(coverage_tool)
        .tool(drift_tool)
        .tool(gaps_tool)
        .tool(simulate_tool)
        .resource(cert_resource)
        .resource_template(context_resource)
        .resource_template(summary_resource);

    info!("ARES MCP Server started on stdio");

    StdioTransport::new(router).run().await?;

    Ok(())
}
