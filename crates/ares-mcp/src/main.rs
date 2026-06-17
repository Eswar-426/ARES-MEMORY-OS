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
    let facade = Arc::new(MemoryFacade::new(assembler.clone()));

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

    // Create Router and attach tools
    let router = McpRouter::new()
        .server_info("ares-mcp", env!("CARGO_PKG_VERSION"))
        .tool(why_tool)
        .tool(who_tool)
        .tool(evolution_tool)
        .tool(impact_tool)
        .resource(cert_resource)
        .resource_template(context_resource);

    info!("ARES MCP Server started on stdio");

    StdioTransport::new(router).run().await?;
    
    Ok(())
}
