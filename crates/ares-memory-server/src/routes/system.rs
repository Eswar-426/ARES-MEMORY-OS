use crate::capabilities::registered_capabilities;
use axum::Json;
use serde_json::{json, Value};

/// GET /health
/// Liveness check. Returns 200 OK if the server is running.
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "ares-memory-server"
    }))
}

/// GET /version
/// Returns the server's crate version and build metadata.
pub async fn version() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "service": env!("CARGO_PKG_NAME"),
        "description": env!("CARGO_PKG_DESCRIPTION")
    }))
}

/// GET /capabilities
/// Returns the runtime registry of all known ARES intelligence capabilities.
/// Used by enterprise deployments for introspection and governance.
pub async fn capabilities() -> Json<Value> {
    let caps = registered_capabilities();
    Json(json!(caps))
}

/// GET /architecture
/// Returns the validated ARES layering model.
/// In a future phase, this will dynamically expose the real dependency graph.
pub async fn architecture() -> Json<Value> {
    Json(json!({
        "layers": [
            { "name": "core",           "crate": "ares-core",           "description": "Fundamental primitives, types, error handling" },
            { "name": "store",          "crate": "ares-store",          "description": "SQLite persistence, repository contracts" },
            { "name": "retrieval",      "crate": "ares-retrieval",      "description": "Memory fetch, graph traversal, search" },
            { "name": "intelligence",   "crate": "(multiple)",          "description": "Decision, lifecycle, bootstrap, gap, repository intelligence" },
            { "name": "orchestration",  "crate": "ares-query",          "description": "Canonical public query surface" },
            { "name": "interface",      "crate": "ares-memory-server",  "description": "HTTP gateway, CLI, MCP, IDE extension" }
        ],
        "dependency_direction": "Core → Store → Retrieval → Intelligence → Orchestration → Interface",
        "gate": "G1",
        "gate_status": "in_progress"
    }))
}
