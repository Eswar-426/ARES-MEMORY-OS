use crate::handler::ToolHandler;
use crate::tools::McpServer;
use ares_app::AppState;
use jsonrpc_core::{IoHandler, Params};
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Write};
use tracing::{debug, error, info};

pub struct StdioServer {
    state: AppState,
    mcp: McpServer,
}

impl StdioServer {
    pub fn new(state: AppState) -> Self {
        let mcp = McpServer::new(state.clone());
        Self { state, mcp }
    }

    pub async fn run(&self) -> Result<(), String> {
        info!("Starting MCP stdio server...");

        let mut io = IoHandler::new();

        // Server info closure
        let server_info = self.mcp.get_info();
        let capabilities = self.mcp.get_capabilities();

        // ─── MCP Lifecycle Methods ─────────────────────────────────

        // 1. initialize
        io.add_method("initialize", move |params: Params| {
            debug!("Received initialize request: {:?}", params);
            let response = serde_json::json!({
                "protocolVersion": server_info.protocol_version,
                "serverInfo": {
                    "name": server_info.name,
                    "version": server_info.version
                },
                "capabilities": capabilities
            });
            async { Ok(response) }
        });

        // 2. notifications/initialized
        io.add_notification("notifications/initialized", |params| {
            debug!("Received initialized notification: {:?}", params);
        });

        // ─── Tools API ─────────────────────────────────────────────

        // 3. tools/list
        let tools_list = self.mcp.list_tools();
        io.add_method("tools/list", move |_params: Params| {
            let tools_value: Vec<Value> = tools_list
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": t.input_schema
                    })
                })
                .collect();

            async move {
                Ok(serde_json::json!({
                    "tools": tools_value
                }))
            }
        });

        // 4. tools/call
        let local_state = self.state.clone();
        io.add_method("tools/call", move |params: Params| -> jsonrpc_core::BoxFuture<Result<serde_json::Value, jsonrpc_core::Error>> {
            let local_state = local_state.clone();
            Box::pin(async move {
                let local_handler = ToolHandler::new(local_state);

                let args: Value = match params.parse() {
                    Ok(v) => v,
                    Err(e) => return Err(jsonrpc_core::Error::invalid_params(format!("{}", e))),
                };

                let tool_name = match args.get("name").and_then(|v| v.as_str()) {
                    Some(name) => name.to_string(),
                    None => return Err(jsonrpc_core::Error::invalid_params("Missing tool name")),
                };

                let tool_args = args.get("arguments").cloned().unwrap_or(serde_json::json!({}));

                debug!(tool = %tool_name, args = ?tool_args, "Executing tool call");

                match local_handler.handle(&tool_name, &tool_args).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        error!(error = %e, "Tool execution failed");
                        let mut rpc_err = jsonrpc_core::Error::internal_error();
                        rpc_err.message = e;
                        Err(rpc_err)
                    }
                }
            })
        });

        // ─── Stdio transport loop ─────────────────────────────────

        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let mut stdout = io::stdout();

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let req_str = line.trim();
                    if req_str.is_empty() {
                        continue;
                    }

                    debug!("Received: {}", req_str);

                    if let Some(response) = io.handle_request_sync(req_str) {
                        debug!("Sending: {}", response);
                        writeln!(stdout, "{}", response).map_err(|e| e.to_string())?;
                        stdout.flush().map_err(|e| e.to_string())?;
                    }
                }
                Err(e) => return Err(e.to_string()),
            }
        }

        Ok(())
    }
}
