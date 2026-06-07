use crate::tools::McpServer;
use jsonrpc_core::{IoHandler, Params, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info, debug};
use std::sync::Arc;

pub async fn run_stdio_server(mcp: Arc<McpServer>) -> anyhow::Result<()> {
    let mut io = IoHandler::new();

    let mcp_clone = mcp.clone();
    io.add_method("server/info", move |_params: Params| {
        let info = mcp_clone.get_info();
        async move { Ok(serde_json::to_value(info).unwrap()) }
    });

    let mcp_clone = mcp.clone();
    io.add_method("tools/list", move |_params: Params| {
        let tools = mcp_clone.list_tools();
        async move { Ok(serde_json::to_value(tools).unwrap()) }
    });

    // Mock implementation for one tool to show wiring
    let _mcp_clone = mcp.clone();
    io.add_method("tools/call", move |params: Params| {
        async move {
            let map = match params {
                Params::Map(m) => m,
                _ => return Err(jsonrpc_core::Error::invalid_params("Expected named parameters")),
            };
            
            let name = map.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let _args = map.get("arguments").unwrap_or(&Value::Null);

            // In a real implementation, we would route to mcp_clone.state engines based on `name`
            Ok(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("Tool {} executed successfully", name)
                    }
                ]
            }))
        }
    });

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    info!("MCP server listening on stdio");

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                debug!("Received: {}", line.trim());
                if let Some(response) = io.handle_request_sync(&line) {
                    debug!("Responding: {}", response);
                    stdout.write_all(response.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    Ok(())
}
