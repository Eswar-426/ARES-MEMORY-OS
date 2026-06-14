use ares_app::AppState;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    pub protocol_version: String,
}

#[derive(Serialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Serialize)]
pub struct McpCapabilities {
    pub tools: McpToolCapability,
}

#[derive(Serialize)]
pub struct McpToolCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
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
            protocol_version: "2024-11-05".into(),
        }
    }

    pub fn get_capabilities(&self) -> McpCapabilities {
        McpCapabilities {
            tools: McpToolCapability {
                list_changed: false,
            },
        }
    }

    pub fn list_tools(&self) -> Vec<McpToolInfo> {
        vec![
            // ── Core Memory Tools ─────────────────────────────────
            McpToolInfo {
                name: "search_memory".into(),
                description: "Search the ARES memory store for project knowledge, decisions, features, and bugs".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query text" },
                        "project_id": { "type": "string", "description": "Optional project ID (uses default if omitted)" },
                        "limit": { "type": "number", "description": "Max results (default: 10)" }
                    },
                    "required": ["query"]
                }),
            },
            McpToolInfo {
                name: "store_memory".into(),
                description: "Store a new memory (decision, feature, bug, or note) in the project memory".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "The memory content to store" },
                        "title": { "type": "string", "description": "Short title for the memory" },
                        "memory_type": {
                            "type": "string",
                            "enum": ["decision", "feature", "bug", "architecture", "project", "agent"],
                            "description": "Type of memory (default: feature)"
                        },
                        "project_id": { "type": "string", "description": "Optional project ID" }
                    },
                    "required": ["content"]
                }),
            },
            McpToolInfo {
                name: "update_memory".into(),
                description: "Update an existing memory with new content (creates a new version)".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Memory ID to update" },
                        "content": { "type": "string", "description": "New content" }
                    },
                    "required": ["id", "content"]
                }),
            },
            // ── Context Tools ─────────────────────────────────────
            McpToolInfo {
                name: "get_project_context".into(),
                description: "Generate portable project context for AI continuity. Use this when switching between AI models to restore full project understanding.".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "Optional project ID" },
                        "max_tokens": { "type": "number", "description": "Optional token budget for context compression" }
                    }
                }),
            },
            McpToolInfo {
                name: "get_context_for_prompt".into(),
                description: "Retrieves AI-ready context packages for a given prompt, including relevant architecture, decisions, and bugs.".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "Optional project ID" },
                        "prompt": { "type": "string", "description": "The user prompt to generate context for" }
                    },
                    "required": ["prompt"]
                }),
            },
            McpToolInfo {
                name: "get_context".into(),
                description: "Get AI-ready context for a specific query".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Context query" }
                    },
                    "required": ["query"]
                }),
            },
            // ── Project Tools ─────────────────────────────────────
            McpToolInfo {
                name: "generate_snapshot".into(),
                description: "Generate and save a complete project snapshot (architecture, languages, dependencies, decisions, features, bugs)".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "Optional project ID" }
                    }
                }),
            },
            McpToolInfo {
                name: "list_projects".into(),
                description: "List all registered projects in ARES".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpToolInfo {
                name: "project_status".into(),
                description: "Get current project status including memory counts and maturity".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // ── Intelligence Tools ────────────────────────────────
            McpToolInfo {
                name: "decision_history".into(),
                description: "Get the history of technical decisions made in the project".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "Optional project ID" }
                    }
                }),
            },
            McpToolInfo {
                name: "detect_contradictions".into(),
                description: "Detect contradictions in project state and decisions".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpToolInfo {
                name: "scan_project".into(),
                description: "Trigger a full project scan to update the knowledge graph".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpToolInfo {
                name: "semantic_search".into(),
                description: "Search memories by semantic meaning using embeddings".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "number" }
                    },
                    "required": ["query"]
                }),
            },
            // ── Orchestration Tools ───────────────────────────────
            McpToolInfo {
                name: "run_workflow".into(),
                description: "Start a workflow execution".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_version_id": { "type": "string" }
                    },
                    "required": ["workflow_version_id"]
                }),
            },
            McpToolInfo {
                name: "workflow_metrics".into(),
                description: "Get analytics and runtime metrics for workflows".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpToolInfo {
                name: "list_agents".into(),
                description: "List registered agents and their health".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpToolInfo {
                name: "create_plan_from_goal".into(),
                description: "Create an autonomous development plan from a high-level goal".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "goal": { "type": "string", "description": "High-level goal statement" },
                        "priority": { "type": "string", "enum": ["Low", "Medium", "High", "Critical"], "description": "Goal priority (default: Medium)" }
                    },
                    "required": ["goal"]
                }),
            },
        ]
    }
}
