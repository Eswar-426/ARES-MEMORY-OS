use crate::agent::{AgentType, ToolResult};
use ares_mcp::handler::ToolHandler;
use serde_json::Value;

pub struct BenchmarkTools {
    pub ares_handler: Option<ToolHandler>,
}

impl BenchmarkTools {
    pub fn new(ares_handler: Option<ToolHandler>) -> Self {
        Self { ares_handler }
    }

    /// Return the list of tools available for a given AgentType.
    pub fn get_schemas(&self, agent_type: AgentType) -> Vec<Value> {
        let mut tools = vec![
            serde_json::json!({
                "name": "read_file",
                "description": "Read the contents of a file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }
            }),
            serde_json::json!({
                "name": "search_codebase",
                "description": "Search for a keyword in the codebase.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }
            }),
            serde_json::json!({
                "name": "write_file",
                "description": "Write contents to a file (simulated in benchmark).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["path", "content"]
                }
            }),
        ];

        // If the agent uses ARES, we inject the ARES tools.
        if agent_type == AgentType::Ares || agent_type == AgentType::ContextDumpAndAres {
            tools.push(serde_json::json!({
                "name": "get_context_for_prompt",
                "description": "Get ARES injected context for a specific task prompt.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string" }
                    },
                    "required": ["prompt"]
                }
            }));
            tools.push(serde_json::json!({
                "name": "search_memory",
                "description": "Search the ARES memory bank for past decisions or architecture.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }
            }));
        }

        tools
    }

    /// Execute a tool by name.
    pub async fn execute(&self, name: &str, args: &Value) -> ToolResult {
        match name {
            "read_file" => {
                // In a real runner, this would read from the isolated workspace
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                ToolResult {
                    output: format!("Contents of {path} (simulated)"),
                    is_error: false,
                }
            }
            "search_codebase" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                ToolResult {
                    output: format!("Search results for {query} (simulated)"),
                    is_error: false,
                }
            }
            "write_file" => {
                ToolResult {
                    output: "File written successfully.".into(),
                    is_error: false,
                }
            }
            "get_context_for_prompt" | "search_memory" => {
                if let Some(handler) = &self.ares_handler {
                    match handler.handle(name, args).await {
                        Ok(res) => ToolResult {
                            output: res.to_string(),
                            is_error: false,
                        },
                        Err(e) => ToolResult {
                            output: format!("ARES Error: {e}"),
                            is_error: true,
                        },
                    }
                } else {
                    ToolResult {
                        output: "ARES not initialized.".into(),
                        is_error: true,
                    }
                }
            }
            _ => ToolResult {
                output: format!("Unknown tool: {name}"),
                is_error: true,
            },
        }
    }
}
