use ares_app::AppState;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct McpServer {
    pub state: AppState,
}

impl McpServer {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn get_info(&self) -> McpServerInfo {
        McpServerInfo {
            name: "ares-mcp".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    pub fn list_tools(&self) -> Vec<McpToolInfo> {
        vec![
            McpToolInfo {
                name: "search_memory".into(),
                description: "Search the ARES memory store".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "number" }
                    },
                    "required": ["query"]
                }),
            },
            McpToolInfo {
                name: "create_memory".into(),
                description: "Create a new memory".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string" }
                    },
                    "required": ["content"]
                }),
            },
            McpToolInfo {
                name: "update_memory".into(),
                description: "Update an existing memory".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["id", "content"]
                }),
            },
            McpToolInfo {
                name: "get_context".into(),
                description: "Get AI-ready context for a query".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }),
            },
            McpToolInfo {
                name: "decision_history".into(),
                description: "Get history for a decision".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "decision_id": { "type": "string" }
                    },
                    "required": ["decision_id"]
                }),
            },
            McpToolInfo {
                name: "detect_contradictions".into(),
                description: "Detect contradictions in project state".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpToolInfo {
                name: "scan_project".into(),
                description: "Trigger a full project scan".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpToolInfo {
                name: "project_status".into(),
                description: "Get current project status".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpToolInfo {
                name: "semantic_search".into(),
                description: "Search memories by semantic meaning".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "number" }
                    },
                    "required": ["query"]
                }),
            },
            McpToolInfo {
                name: "reason_about_query".into(),
                description: "Run reasoning pipeline for a query to get evidence-backed context"
                    .into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }),
            },
            McpToolInfo {
                name: "explain_decision".into(),
                description: "Explain why a decision was made and its evolution".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "decision_id": { "type": "string" }
                    },
                    "required": ["decision_id"]
                }),
            },
            McpToolInfo {
                name: "impact_analysis".into(),
                description: "Analyze the impact of changing a node".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "string" },
                        "depth": { "type": "number" }
                    },
                    "required": ["node_id"]
                }),
            },
            McpToolInfo {
                name: "dependency_graph".into(),
                description: "Get transitive dependencies for a node".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "string" }
                    },
                    "required": ["node_id"]
                }),
            },
            McpToolInfo {
                name: "decision_timeline".into(),
                description: "Get the lineage and timeline of a decision".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "decision_id": { "type": "string" }
                    },
                    "required": ["decision_id"]
                }),
            },
            McpToolInfo {
                name: "memory_timeline".into(),
                description: "Get the full timeline of a memory".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_id": { "type": "string" }
                    },
                    "required": ["memory_id"]
                }),
            },
            McpToolInfo {
                name: "contradiction_analysis".into(),
                description: "Deeply analyze a detected contradiction".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "contradiction_ids": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["contradiction_ids"]
                }),
            },
        ]
    }
}
