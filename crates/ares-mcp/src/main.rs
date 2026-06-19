use ares_agent::config::AgentConfig;
use ares_app::AppState;
use ares_memory_intelligence::assembler::MemoryContextAssembler;
use ares_memory_intelligence::facade::MemoryFacade;
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tower_mcp::{protocol::{CallToolResult, ReadResourceResult}, router::McpRouter, tool::ToolBuilder, resource::{ResourceBuilder, ResourceTemplateBuilder}, transport::stdio::StdioTransport, BoxError};
use tracing::info;

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

    let assembler = Arc::new(MemoryContextAssembler::default_from_store(app_state.store.clone()));
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
                match facade.why(&input.id) {
                    Ok(result) => Ok(CallToolResult::text(result.to_string())),
                    Err(e) => Ok(CallToolResult::text(format!("Error: {}", e))),
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
                match facade.who(&input.id) {
                    Ok(result) => Ok(CallToolResult::text(result.to_string())),
                    Err(e) => Ok(CallToolResult::text(format!("Error: {}", e))),
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
                match facade.evolution(&input.id) {
                    Ok(result) => Ok(CallToolResult::text(serde_json::to_string(&result).unwrap_or_else(|_| "Failed to serialize".to_string()))),
                    Err(e) => Ok(CallToolResult::text(format!("Error: {}", e))),
                }
            }
        })
        .build();

    // Create the Impact tool
    let facade_impact = facade.clone();
    let impact_tool = ToolBuilder::new("ares_impact")
        .description("Analyzes the impact of changing or removing an entity")
        .handler(move |input: MemoryQueryInput| {
            let facade = facade_impact.clone();
            async move {
                match facade.impact(&input.id) {
                    Ok(result) => Ok(CallToolResult::text(serde_json::to_string(&result).unwrap_or_else(|_| "Failed to serialize".to_string()))),
                    Err(e) => Ok(CallToolResult::text(format!("Error: {}", e))),
                }
            }
        })
        .build();

    // Create the Certification Resource
    let cert_runner = Arc::new(ares_validation::validation_runner::ValidationRunner::new(
        Arc::new(app_state.store.clone()),
        assembler.clone()
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
                    Ok(result) => Ok(ReadResourceResult::text("memory://certification", serde_json::to_string(&result).unwrap_or_default())),
                    Err(e) => Err(tower_mcp::Error::internal(e.to_string())),
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
            let id = vars.get("id").cloned().unwrap_or_default();
            async move {
                match facade.context(&id) {
                    Ok(result) => Ok(ReadResourceResult::text(uri, serde_json::to_string(&result).unwrap_or_default())),
                    Err(e) => Err(tower_mcp::Error::internal(e.to_string())),
                }
            }
        });

    // Create the Compliance tool
    let facade_compliance = facade.clone();
    let compliance_tool = ToolBuilder::new("ares_compliance")
        .description("Evaluates the compliance of a specific entity against active governance policies")
        .handler(move |input: GovernanceQueryInput| {
            let facade = facade_compliance.clone();
            async move {
                let governance = facade.get_governance();
                match governance.is_compliant(&ares_core::ProjectId::from(input.project_id), &ares_core::NodeId::from(input.node_id)).await {
                    Ok(result) => Ok(CallToolResult::text(serde_json::to_string(&result).unwrap_or_else(|_| "Failed to serialize".to_string()))),
                    Err(e) => Ok(CallToolResult::text(format!("Error: {}", e))),
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
                match governance.get_scorecard(&ares_core::ProjectId::from(input.project_id)).await {
                    Ok(result) => Ok(CallToolResult::text(serde_json::to_string(&result).unwrap_or_else(|_| "Failed to serialize".to_string()))),
                    Err(e) => Ok(CallToolResult::text(format!("Error: {}", e))),
                }
            }
        })
        .build();

    // Create Router and attach tools
    let router = McpRouter::new()
        .server_info("ares-mcp", env!("CARGO_PKG_VERSION"))
        .tool(why_tool)
        .tool(who_tool)
        .tool(evolution_tool)
        .tool(impact_tool)
        .tool(compliance_tool)
        .tool(scorecard_tool)
        .resource(cert_resource)
        .resource_template(context_resource);

    info!("ARES MCP Server started on stdio");

    StdioTransport::new(router).run().await?;
    
    Ok(())
}
