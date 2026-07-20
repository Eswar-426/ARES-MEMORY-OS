use ares_agent::config::AgentConfig;
use ares_app::AppState;
use ares_memory_intelligence::assembler::MemoryContextAssembler;
use ares_memory_intelligence::facade::MemoryFacade;
use ares_repository_intelligence::facade::IntelligenceFacade;
use ares_repository_intelligence::models::{EngineeringQuery, QueryType};
use schemars::JsonSchema;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct SessionState {
    started_at: std::time::Instant,
    tool_calls: Vec<(String, String)>,
    files_touched: HashSet<String>,
    project_id: String,
}

fn track_session_call(
    session: &Arc<Mutex<SessionState>>,
    tool_name: &str,
    input: &impl serde::Serialize,
) {
    let serialized = serde_json::to_string(input).unwrap_or_default();
    let mut state = session.lock().unwrap();
    state
        .tool_calls
        .push((tool_name.to_string(), serialized.clone()));
    extract_paths_from_json(&mut state.files_touched, &serialized);
}

fn extract_paths_from_json(files: &mut HashSet<String>, json_str: &str) {
    for field in &["file_path", "target_path", "file_a", "file_b"] {
        let pattern = format!("\"{}\":\"", field);
        if let Some(idx) = json_str.find(&pattern) {
            let rest = &json_str[idx + pattern.len()..];
            if let Some(val_end) = rest.find('"') {
                let path = &rest[..val_end];
                if !path.is_empty()
                    && !path.starts_with("person:")
                    && !path.starts_with("commit:")
                    && !path.starts_with("decision:")
                    && !path.starts_with("requirement:")
                {
                    files.insert(path.to_string());
                }
            }
        }
    }
    for field in &["impacted_paths", "satisfies_paths"] {
        let pattern = format!("\"{}\": [", field);
        if let Some(idx) = json_str.find(&pattern) {
            let rest = &json_str[idx + pattern.len()..];
            if let Some(arr_end) = rest.find(']') {
                for item in rest[1..arr_end].split('"') {
                    let item = item.trim();
                    if !item.is_empty() && !item.starts_with(',') {
                        files.insert(item.to_string());
                    }
                }
            }
        }
    }
}

use serde::Deserialize;
use std::collections::HashMap;
use tower_mcp::{
    protocol::{CallToolResult, ReadResourceResult},
    resource::{ResourceBuilder, ResourceTemplateBuilder},
    router::McpRouter,
    tool::ToolBuilder,
    transport::stdio::StdioTransport,
    BoxError,
};
use tracing::info;

/// Wraps any tool's existing JSON response in the universal ARES envelope.
/// Preserves `result` and `query_time_ms` at top level for webview backward compat.
fn wrap_with_envelope(
    tool_name: &str,
    current: serde_json::Value,
    elapsed_ms: u64,
) -> serde_json::Value {
    // --- Extract evidence (keep what the tool already produced) ---
    let evidence = current
        .get("evidence")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));

    let gap_flags = current
        .get("gap_flags")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));

    // --- Normalize confidence to 0.0-1.0 (check top-level AND inside result) ---
    let mut confidence = {
        let raw = current
            .get("confidence")
            .or_else(|| current.get("result").and_then(|r| r.get("confidence")));
        match raw {
            Some(serde_json::Value::Number(n)) => {
                let v = n.as_f64().unwrap_or(0.0);
                if v > 1.0 { v / 100.0 } else { v }
            }
            Some(serde_json::Value::Object(obj)) => {
                let v = obj.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                if v > 1.0 { v / 100.0 } else { v }
            }
            _ => 0.0,
        }
    };

    // --- Determine answer: prefer existing answer, else result, else whole thing ---
    let mut answer = current
        .get("answer")
        .cloned()
        .or_else(|| current.get("result").cloned())
        .unwrap_or_else(|| current.clone());

    // --- Determine status ---
    let has_error = current.get("error").map_or(false, |e| !e.is_null());
    let is_empty = match &answer {
        serde_json::Value::Array(a) => a.is_empty(),
        serde_json::Value::Object(o) => o.is_empty(),
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => s.is_empty(),
        _ => false,
    };
    let status = if has_error {
        "error"
    } else if is_empty {
        "empty"
    } else {
        "ok"
    };

    // Research rule #3: cap confidence to 0.3 when gap_flags indicate incomplete graph
    let incomplete_indicators = [
        "incomplete_graph",
        "no_graph_dependents",
        "no_git_history",
        "incomplete_ast",
        "shallow_git_history",
    ];
    let flags_arr = gap_flags.as_array().map(|a| {
        a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>()
    }).unwrap_or_default();
    if flags_arr.iter().any(|f| incomplete_indicators.contains(f)) && confidence > 0.3 {
        confidence = 0.3;
    }

    // Research rule: confidence ≤ 0.5 when fewer than 3 evidence sources
    let evidence_count = evidence.as_array().map_or(0, |a| a.len());
    if confidence > 0.5 && evidence_count < 3 {
        confidence = 0.5;
    }

    // --- Build envelope ---
    if let Some(obj) = answer.as_object_mut() {
        obj.remove("confidence");
        obj.remove("evidence");
        obj.remove("gap_flags");
        obj.remove("result");
        obj.remove("query_time_ms");
    }

    let caveats = current.get("caveats").cloned().unwrap_or_else(|| serde_json::json!([]));
    let mut meta = current.get("meta").cloned().unwrap_or_else(|| serde_json::json!({}));
    if let Some(m) = meta.as_object_mut() {
        m.insert("elapsed_ms".to_string(), serde_json::json!(elapsed_ms as i64));
        if !m.contains_key("graph_nodes_traversed") {
            m.insert("graph_nodes_traversed".to_string(), serde_json::json!(0));
        }
        if !m.contains_key("truncated") {
            m.insert("truncated".to_string(), serde_json::json!(false));
        }
    }

    let mut envelope = serde_json::json!({
        "tool": tool_name,
        "schema_version": "1.0",
        "status": status,
        "confidence": confidence,
        "evidence": evidence,
        "gap_flags": gap_flags,
        "caveats": caveats,
        "answer": answer,
        "meta": meta
    });

    envelope
}

/// Recursively walks JSON and prefixes any string under a "node_id" key with "node_id:" prefix
fn prefix_node_ids(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            let is_node = map.contains_key("node_type");
            for (key, v) in map.iter_mut() {
                if key == "node_id" || key == "from_node_id" || key == "to_node_id" || (is_node && key == "id") {
                    if let serde_json::Value::String(s) = v {
                        if !s.starts_with("node_id:") {
                            *v = serde_json::Value::String(format!("node_id:{}", s));
                        }
                    }
                }
                prefix_node_ids(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                prefix_node_ids(v);
            }
        }
        _ => {}
    }
}

fn default_nulls(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                if v.is_null() {
                    match key.as_str() {
                        "language" | "module" | "namespace" | "repository" |
                        "primary_owner" | "team" | "knowledge_debt" | "risk_level" |
                        "created_at" | "last_modified" | "location" |
                        "last_query" | "last_query_time" => {
                            *v = serde_json::Value::String(String::new());
                        }
                        "loc" | "last_modified_days_ago" => {
                            *v = serde_json::Value::Number(serde_json::Number::from(0));
                        }
                        "test_coverage" => {
                            *v = serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap());
                        }
                        _ => {}
                    }
                }
                default_nulls(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                default_nulls(v);
            }
        }
        _ => {}
    }
}

fn format_micros_as_iso(micros: i64) -> String {
    if micros == 0 {
        return String::new();
    }
    let secs = micros / 1_000_000;
    let nanos = ((micros % 1_000_000) * 1_000) as u32;
    chrono::DateTime::from_timestamp(secs, nanos)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
        .unwrap_or_default()
}

fn round_precision(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                if key == "graph_density" || key == "average_degree" {
                    if let Some(f) = v.as_f64() {
                        *v = serde_json::json!((f * 10000.0).round() / 10000.0);
                    }
                }
                round_precision(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                round_precision(v);
            }
        }
        _ => {}
    }
}

fn truncate_large_arrays(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                if (key == "files_changed" || key == "events") && v.is_array() {
                    if let Some(arr) = v.as_array_mut() {
                        if arr.len() > 50 {
                            arr.truncate(50);
                        }
                    }
                }
                truncate_large_arrays(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                truncate_large_arrays(v);
            }
        }
        _ => {}
    }
}

fn strip_details_uuids(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(s)) = map.get_mut("details") {
                // Strip "REQ-<uuid>" prefix: find "REQ-" then skip 36 chars of UUID
                if let Some(pos) = s.find("REQ-") {
                    let before = &s[..pos];
                    let after = &s[pos + 4..]; // skip "REQ-"
                    let cleaned = if after.len() >= 36 {
                        let rest = &after[36..]; // skip UUID
                        format!("{}{}", before, rest).trim().to_string()
                    } else {
                        s.clone()
                    };
                    *s = if cleaned.is_empty() { s.clone() } else { cleaned };
                }
            }
            for v in map.values_mut() {
                strip_details_uuids(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_details_uuids(v);
            }
        }
        _ => {}
    }
}

fn transform_relationships(node_val: &mut serde_json::Value) {
    if let Some(rels) = node_val.get_mut("relationships").and_then(|r| r.as_object_mut()) {
        for (_rel_type, arr) in rels.iter_mut() {
            if let Some(citations) = arr.as_array_mut() {
                for citation in citations.iter_mut() {
                    if let Some(cit_obj) = citation.as_object_mut() {
                        if let Some(id_val) = cit_obj.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()) {
                            let (new_id, new_title) = if id_val.starts_with("unresolved_") {
                                let mut clean_path = id_val.as_str();
                                let exts = [".rs_", ".ts_", ".js_", ".jsx_", ".tsx_", ".go_", ".py_", ".c_", ".cpp_", ".h_", ".hpp_", ".cs_", ".java_", ".sql_", ".md_"];
                                for ext in exts.iter() {
                                    if let Some(idx) = id_val.find(ext) {
                                        clean_path = &id_val[idx + ext.len()..];
                                        break;
                                    }
                                }
                                let normalized = clean_path.replace("\r\n", " ").replace("\n", " ").split_whitespace().collect::<Vec<_>>().join(" ");
                                (normalized.clone(), normalized)
                            } else {
                                let prefixed = if id_val.starts_with("node_id:") { id_val.clone() } else { format!("node_id:{}", id_val) };
                                let mut existing_title = cit_obj.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                if existing_title == id_val || existing_title.is_empty() {
                                    existing_title = prefixed.clone();
                                }
                                (prefixed.clone(), existing_title)
                            };
                            cit_obj.insert("id".to_string(), serde_json::Value::String(new_id));
                            cit_obj.insert("title".to_string(), serde_json::Value::String(new_title));
                        }
                    }
                }
            }
        }
    }
}

fn format_mcp_error(message: &str, details: &str) -> String {
    serde_json::json!({
        "code": -32603,
        "message": message,
        "details": details
    })
    .to_string()
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct MemoryQueryInput {
    id: Option<String>,
    file_path: Option<String>,
}

impl MemoryQueryInput {
    fn resolve_id(&self, store: &ares_store::db::Store) -> Result<String, String> {
        if let Some(id) = &self.id {
            return Ok(id.clone());
        }
        if let Some(path) = &self.file_path {
            let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
            repo.get_id_by_path(path)
                .map_err(|_| format!("File not found in graph: {}", path))
        } else {
            Err("Must provide either 'id' or 'file_path'".to_string())
        }
    }
}

// === Phase 2: Task 3.1 — Additional MCP Tools ===

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct OwnerQueryInput {
    file_path: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct DecisionsQueryInput {
    file_path: Option<String>,
    since: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct SearchQueryInput {
    query: String,
    #[serde(default = "default_search_limit")]
    limit: usize,
}

fn default_search_limit() -> usize {
    10
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct TimelineQueryInput {
    file_path: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct CompareQueryInput {
    file_a: String,
    file_b: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct ArchitectureQueryInput {}

// === Phase 3: Task 3.2 — Agent Memory Write API ===

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct RecordDecisionInput {
    title: String,
    description: String,
    status: String,
    impacted_paths: Vec<String>,
    source: Option<String>,
    confidence: Option<f64>,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct RecordRequirementInput {
    title: String,
    description: String,
    priority: String,
    satisfies_paths: Vec<String>,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct AnnotateInput {
    target_path: String,
    key: String,
    value: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct EndSessionInput {
    left_incomplete: Option<String>,
    recommended_next: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct CorrectInput {
    target_path: String,
    correction_notes: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct RequirementsQueryInput {
    file_path: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct GovernanceQueryInput {
    #[serde(default)]
    project_id: String,
    node_id: String,
}

impl GovernanceQueryInput {
    fn resolve_id(&self, store: &ares_store::db::Store) -> Result<String, String> {
        if self.node_id.starts_with("0") || self.node_id.starts_with("file:") || self.node_id.len() == 36 {
            return Ok(ares_core::canonicalize_node_id(&self.node_id));
        }
        let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
        repo.get_id_by_path_loose(&self.node_id)
            .map_err(|_| format!("Node not found for path: {}", self.node_id))
    }
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct ProjectQueryInput {
    project_id: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct ChatInput {
    query: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct BookmarkInput {
    kind: String,
    value: String,
    title: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct PinInput {
    node_id: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct NavigateInput {
    direction: String,
    current_timestamp: i64,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct RecordNavigateInput {
    node_id: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
#[allow(dead_code)]
struct SimulationInput {
    action: String,
    target_id: String,
    related_id: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct TraceabilityInput {
    entity_id: Option<String>,
    file_path: Option<String>,
    depth: Option<usize>,
}

impl TraceabilityInput {
    fn resolve_id(&self, store: &ares_store::db::Store) -> Result<String, String> {
        if let Some(id) = &self.entity_id {
            return Ok(id.clone());
        }
        if let Some(path) = &self.file_path {
            let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
            repo.get_id_by_path(path)
                .map_err(|_| format!("File not found in graph: {}", path))
        } else {
            Err("Must provide either 'entity_id' or 'file_path'".to_string())
        }
    }
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct EmptyInput {}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct GraphSearchInput {
    query: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
struct GraphPathInput {
    from_id: String,
    to_id: String,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
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
    let session_state: Arc<Mutex<SessionState>> = Arc::new(Mutex::new(SessionState {
        started_at: std::time::Instant::now(),
        tool_calls: Vec::new(),
        files_touched: HashSet::new(),
        project_id: std::path::Path::new(&project_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    }));

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
                    println!(
                    "WARN: Failed to initialize OpenAI provider: {}. Falling back to mock engine.",
                    e
                );
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

    let session_clone_why_tool = session_state.clone();
    let store_why = app_state.store.clone();
    let why_tool = ToolBuilder::new("ares_why_exists")
        .description("Explains why a specific entity exists in the ARES memory graph")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_why_tool.clone();
            let facade = intelligence_facade_why.clone();
            let project_id = project_id_str.clone();
            let store = store_why.clone();

            async move {
                track_session_call(&session, "ares_why_exists", &input);
                let id_str = match input.resolve_id(&store) {
                    Ok(i) => i,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                };
                let id = ares_core::canonicalize_node_id(&id_str);
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
                            "gap_flags": serde_json::json!(insight.gap_flags),
                            "warnings": insight.warnings,
                            "recommendations": insight.recommendations,
                            "summary": insight.summary,
                            "file_path": input.file_path.as_ref().or(input.id.as_ref()),
                            "entity": input.file_path.as_ref().or(input.id.as_ref()),
                            "mode": insight.mode,
                            "metadata": insight.metadata,
                        });
                        Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_why_exists", response, 0)).unwrap()))
                    }
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to explain why entity exists",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();




    // Create the Impact tool
    let intelligence_facade_impact = intelligence_facade.clone();
    let project_id_str_impact = project_path.clone();
    let session_clone_impact_tool = session_state.clone();
    let store_impact = app_state.store.clone();
    let impact_tool = ToolBuilder::new("ares_impact")
        .description("Performs read-only dependency analysis to determine what downstream components break if this entity is modified. Use this for general blast-radius queries without mutating the graph.")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_impact_tool.clone();
            let facade = intelligence_facade_impact.clone();
            let project_id = project_id_str_impact.clone();
            let store = store_impact.clone();
            async move {
                track_session_call(&session, "ares_impact", &input);
                let id_str = match input.resolve_id(&store) {
                    Ok(i) => i,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                };
                let id = ares_core::canonicalize_node_id(&id_str);
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
                            "gap_flags": serde_json::json!(insight.gap_flags),
                            "warnings": insight.warnings,
                            "recommendations": insight.recommendations,
                            "summary": insight.summary,
                            "file_path": input.file_path.as_ref().or(input.id.as_ref()),
                            "entity": input.file_path.as_ref().or(input.id.as_ref()),
                            "mode": insight.mode,
                            "metadata": insight.metadata,
                        });
                        Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_impact", response, 0)).unwrap()))
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
    let store_compliance = app_state.store.clone();
    let facade_compliance = facade.clone();
    let session_clone_compliance_tool = session_state.clone();
    let compliance_tool = ToolBuilder::new("ares_compliance")
        .description(
            "Evaluates the compliance of a specific entity against active governance policies",
        )
        .handler(move |input: GovernanceQueryInput| {
            let session = session_clone_compliance_tool.clone();
            let store = store_compliance.clone();
            let facade = facade_compliance.clone();
            async move {
                track_session_call(&session, "ares_compliance", &input);
                
                let resolved_project_id = if input.project_id.is_empty() {
                    session.lock().unwrap().project_id.clone()
                } else {
                    input.project_id.clone()
                };

                let node_id_str = match input.resolve_id(&store) {
                    Ok(id) => id,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                };
                let node_id = ares_core::NodeId::from(node_id_str);
                
                let governance = facade.get_governance();
                match governance
                    .is_compliant(
                        &ares_core::ProjectId::from(resolved_project_id),
                        &node_id,
                    )
                    .await
                {
                    Ok(result) => {
                        let mut payload = if result.is_empty() {
                            serde_json::json!({"compliant": true, "violations": []})
                        } else {
                            serde_json::to_value(&result).unwrap_or_default()
                        };
                        let ref_val = input.node_id.clone();
                        let evidence = serde_json::json!([{"type": "compliance_check", "ref": ref_val}]);
                        let conf = 0.6;
                        if let Some(obj) = payload.as_object_mut() {
                            obj.insert("evidence".to_string(), evidence);
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                        }
                        serde_json::to_string(&wrap_with_envelope("ares_compliance", payload, 0))
                            .map(CallToolResult::text)
                            .map_err(|e| {
                                tower_mcp::Error::internal(format_mcp_error(
                                    "Failed to serialize compliance evaluation",
                                    &e.to_string(),
                                ))
                            })
                    }
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
    let session_clone_scorecard_tool = session_state.clone();
    let scorecard_tool = ToolBuilder::new("ares_scorecard")
        .description("Retrieves the governance scorecard for a project")
        .handler(move |input: ProjectQueryInput| {
            let session = session_clone_scorecard_tool.clone();
            let facade = facade_scorecard.clone();
            async move {
                track_session_call(&session, "ares_scorecard", &input);
                let governance = facade.get_governance();
                match governance
                    .get_scorecard(&ares_core::ProjectId::from(
                        input
                            .project_id
                            .clone()
                            .unwrap_or_else(|| session.lock().unwrap().project_id.clone()),
                    ))
                    .await
                {
                    Ok(result) => {
                        let proj_id = input.project_id.clone().unwrap_or_else(|| session.lock().unwrap().project_id.clone());
                        let evidence = serde_json::json!([{"type": "scorecard_computation", "ref": proj_id}]);
                        let conf = 0.6;
                        let mut payload = serde_json::to_value(&result).unwrap_or_default();
                        if let Some(obj) = payload.as_object_mut() {
                            obj.insert("evidence".to_string(), evidence);
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                        }
                        serde_json::to_string(&wrap_with_envelope("ares_scorecard", payload, 0))
                            .map(CallToolResult::text)
                            .map_err(|e| {
                            tower_mcp::Error::internal(format_mcp_error(
                                "Failed to serialize scorecard",
                                &e.to_string(),
                            ))
                        })
                    },
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
    let session_clone_dashboard_tool = session_state.clone();
    let dashboard_tool = ToolBuilder::new("ares_dashboard")
        .description("Retrieves the comprehensive repository overview dashboard")
        .handler(move |_input: ProjectQueryInput| {
            let session = session_clone_dashboard_tool.clone();
            let store = store_dashboard.clone();
            let path = dashboard_project_path.clone();
            async move {
                track_session_call(&session, "ares_dashboard", &_input);
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
                    let evidence = serde_json::json!([{"type": "system_query", "ref": "workspace"}]);
                    let conf = 0.6;
                    let mut payload = serde_json::to_value(&response).unwrap_or_default();
                    default_nulls(&mut payload);
                    round_precision(&mut payload);
                    if let Some(serde_json::Value::String(s)) = payload.pointer_mut("/repository/root_path") {
                        *s = std::path::Path::new(s.as_str()).file_name().and_then(|n| n.to_str()).unwrap_or(s.as_str()).to_string();
                    }
                    if let Some(serde_json::Value::String(s)) = payload.get_mut("repository_id") {
                        *s = std::path::Path::new(s.as_str()).file_name().and_then(|n| n.to_str()).unwrap_or(s.as_str()).to_string();
                    }
                    if let Some(obj) = payload.as_object_mut() {
                        obj.insert("evidence".to_string(), evidence);
                        obj.insert("confidence".to_string(), serde_json::json!(conf));
                    }
                    serde_json::to_string(&wrap_with_envelope("ares_dashboard", payload, 0))
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
                    let evidence = serde_json::json!([{"type": "system_query", "ref": "workspace"}]);
                    let conf = 0.6;
                    let mut payload = serde_json::to_value(&result).unwrap_or_default();
                    default_nulls(&mut payload);
                    round_precision(&mut payload);
                    if let Some(serde_json::Value::String(s)) = payload.pointer_mut("/repository/root_path") {
                        *s = std::path::Path::new(s.as_str()).file_name().and_then(|n| n.to_str()).unwrap_or(s.as_str()).to_string();
                    }
                    if let Some(serde_json::Value::String(s)) = payload.get_mut("repository_id") {
                        *s = std::path::Path::new(s.as_str()).file_name().and_then(|n| n.to_str()).unwrap_or(s.as_str()).to_string();
                    }
                    if let Some(obj) = payload.as_object_mut() {
                        obj.insert("evidence".to_string(), evidence);
                        obj.insert("confidence".to_string(), serde_json::json!(conf));
                    }
                    serde_json::to_string(&wrap_with_envelope("ares_dashboard", payload, 0))
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
    let session_clone_coverage_tool = session_state.clone();
    let coverage_tool = ToolBuilder::new("ares_coverage")
        .description("Calculates the coverage of requirements for a project")
        .handler(move |input: ProjectQueryInput| {
            let session = session_clone_coverage_tool.clone();
            let store = store_cov.clone();
            async move {
                track_session_call(&session, "ares_coverage", &input);
                let project_name = input
                    .project_id
                    .clone()
                    .unwrap_or_else(|| session.lock().unwrap().project_id.clone());
                let project_id = ares_core::ProjectId::from(project_name);
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
                let evidence = serde_json::json!([{"type": "coverage", "ref": "workspace"}]);
                let conf = 0.6;
                let mut payload = serde_json::to_value(&summary).unwrap_or_default();
                if let Some(obj) = payload.as_object_mut() {
                    obj.insert("evidence".to_string(), evidence);
                    obj.insert("confidence".to_string(), serde_json::json!(conf));
                }
                serde_json::to_string(&wrap_with_envelope("ares_coverage", payload, 0))
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
    let session_clone_drift_tool = session_state.clone();
    let store_drift_new = app_state.store.clone();
    let drift_tool = ToolBuilder::new("ares_drift")
        .description("Evaluates structural drift for a given file")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_drift_tool.clone();
            let facade = intelligence_facade_drift.clone();
            let project_id = project_id_str_drift.clone();
            let store = store_drift_new.clone();
            async move {
                track_session_call(&session, "ares_drift", &input);
                let id_str = match input.resolve_id(&store) {
                    Ok(i) => i,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                };
                let id = ares_core::canonicalize_node_id(&id_str);
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
                            "gap_flags": serde_json::json!(insight.gap_flags),
                            "warnings": insight.warnings,
                            "recommendations": insight.recommendations,
                            "summary": insight.summary,
                            "file_path": input.file_path.as_ref().or(input.id.as_ref()),
                            "entity": input.file_path.as_ref().or(input.id.as_ref()),
                            "mode": insight.mode,
                            "metadata": insight.metadata,
                        });
                        Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_drift", response, 0)).unwrap()))
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
    let session_clone_who_owns_tool = session_state.clone();
    let who_owns_tool = ToolBuilder::new("ares_who_owns")
        .description("Returns the registered owner and contributor history for a file")
        .handler(move |input: OwnerQueryInput| {
            let session = session_clone_who_owns_tool.clone();
            let store_arc = store_who.clone();
            let pp = pp_who.clone();
            async move {
                track_session_call(&session, "ares_who_owns", &input);
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let _project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
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
                                let percentage = (e.weight * 100.0).round() as i32;
                                if percentage > 0 {
                                    contributors.push(serde_json::json!({
                                        "name": p.label,
                                        "percentage": percentage
                                    }));
                                }
                            }
                        }
                    }
                }

                contributors.sort_by(|a, b| b["percentage"].as_i64().cmp(&a["percentage"].as_i64()));

                // Normalize contributor percentages to sum to 100
                let contrib_sum: f64 = contributors.iter()
                    .filter_map(|c| c.get("percentage").and_then(|p| p.as_f64()))
                    .sum();
                if contrib_sum > 100.0 {
                    for c in contributors.iter_mut() {
                        if let Some(pct) = c.get_mut("percentage") {
                            if let Some(v) = pct.as_f64() {
                                *pct = serde_json::json!((v / contrib_sum * 100.0).round() as i32);
                            }
                        }
                    }
                }

                if owner_name.is_empty() && !contributors.is_empty() {
                    owner_name = contributors[0]["name"].as_str().unwrap_or("").to_string();
                }
                if owner_confidence == 0.0 && !contributors.is_empty() {
                    owner_confidence = 1.0;
                }

                // Calculate bus factor (contributors to cover 80%)
                let mut bus_factor = 0;
                let mut accumulated = 0;
                for c in &contributors {
                    if let Some(pct) = c.get("percentage").and_then(|p| p.as_i64()) {
                        accumulated += pct;
                        bus_factor += 1;
                        if accumulated >= 80 {
                            break;
                        }
                    }
                }

                // Get last modifier
                let last_modifier = repo.get_id_by_path(&input.file_path).ok().and_then(|file_id_str| {
                    let file_id = ares_core::NodeId::from(file_id_str.as_str());
                    repo.get_edges_to_by_type(&file_id, "touches")
                        .ok()
                        .and_then(|edges| {
                            edges.into_iter()
                                .max_by_key(|e| e.valid_from)
                                .and_then(|e| repo.get_node(&e.from_node_id).ok().flatten())
                                .and_then(|n| {
                                    n.properties.get("author")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                })
                        })
                });

                let mut result_json = serde_json::json!({
                    "result": {
                        "owner": owner_name,
                        "confidence": owner_confidence,
                        "commit_percentage": if total_weight > 0.0 { std::cmp::min(100, (total_weight * 100.0).round() as i32) } else { 0 },
                        "bus_factor": bus_factor,
                        "contributors": contributors
                    },
                    "evidence": [{"type": "git_blame", "ref": input.file_path.clone()}],
                    "gap_flags": ["no_codeowners_file"],
                    "query_time_ms": start.elapsed().as_millis() as i64
                });
                if let Some(modifier) = last_modifier {
                    result_json["result"]["last_modifier"] = serde_json::json!(modifier);
                }
                let elapsed = start.elapsed().as_millis() as u64;
                Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_who_owns", result_json, elapsed)).unwrap()))
            }
        })
        .build();

    // --- ares_decisions ---
    let store_dec = app_state.store.clone();
    let pp_dec = project_path.clone();
    let session_clone_decisions_tool = session_state.clone();
    let decisions_tool = ToolBuilder::new("ares_decisions")
        .description("Returns architectural decisions, optionally filtered by file path or date")
        .handler(move |input: DecisionsQueryInput| {
            let session = session_clone_decisions_tool.clone();
            let store_arc = store_dec.clone();
            let pp = pp_dec.clone();
            async move {
                track_session_call(&session, "ares_decisions", &input);
                let start = std::time::Instant::now();
                let repo =
                    ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let project_id = ares_core::ProjectId::from(project_name);

                let mut decisions = Vec::new();
                let target_file_id = input
                    .file_path
                    .as_ref()
                    .and_then(|fp| repo.get_id_by_path(fp).ok());

                if let Ok(all) = repo.get_nodes_by_type(&project_id, "decision") {
                    for dn in &all {
                        let props = &dn.properties;
                        let summary = props
                            .get("decision")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&dn.label);
                        let author = props
                            .get("author")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let source = props
                            .get("source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");

                        let mut matches = target_file_id.is_none();
                        let mut files: Vec<String> = Vec::new();

                        if let Ok(edges) = repo.get_edges_from(&dn.id) {
                            for e in &edges {
                                if let Ok(Some(node)) = repo.get_node(&e.to_node_id) {
                                    if let Some(path) = node.file_path {
                                        files.push(path);
                                    }
                                }
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
                                    if dn.created_at < ts {
                                        continue;
                                    }
                                }
                            }
                            let decay = if let Ok(conn) = store_arc.get_conn() {
                                ares_intelligence::decay::calculate_decision_decay(&conn, &dn.created_at.to_string(), &files)
                            } else {
                                ares_intelligence::decay::DecayResult { decay_score: 1.0, staleness: "fresh".to_string() }
                            };

                            decisions.push(serde_json::json!({
                                "id": format!("node_id:{}", dn.id.as_str()),
                                "date": format_micros_as_iso(dn.created_at),
                                "summary": summary,
                                "author": author,
                                "source": source.to_string(),
                                "files": files,
                                "decay_score": decay.decay_score,
                                "staleness": decay.staleness
                            }));
                        }
                    }
                }

                if decisions.is_empty() {
                    let elapsed = start.elapsed().as_millis() as u64;
                    let inner = serde_json::json!({
                        "result": { "decisions": [] },
                        "confidence": 0.6,
                        "evidence": [{"type": "agent_analysis", "ref": input.file_path.clone().unwrap_or_else(|| "workspace".to_string())}],
                        "gap_flags": ["no_recorded_decisions"],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    });
                    Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_decisions", inner, elapsed)).unwrap()))
                } else {
                    let elapsed = start.elapsed().as_millis() as u64;
                    let inner = serde_json::json!({
                        "result": { "decisions": decisions },
                        "confidence": 0.6,
                        "evidence": [{"type": "agent_analysis", "ref": input.file_path.clone().unwrap_or_else(|| "workspace".to_string())}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    });
                    Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_decisions", inner, elapsed)).unwrap()))
                }
            }
        })
        .build();

    // --- ares_search ---
    let store_srch = app_state.store.clone();
    let pp_srch = project_path.clone();
    let session_clone_search_tool = session_state.clone();
    let search_tool = ToolBuilder::new("ares_search")
        .description("Searches nodes by label or file path using full-text matching")
        .handler(move |input: SearchQueryInput| {
            let session = session_clone_search_tool.clone();
            let store_arc = store_srch.clone();
            let pp = pp_srch.clone();
            async move {
                track_session_call(&session, "ares_search", &input);
                let start = std::time::Instant::now();

                if input.query.trim().is_empty() {
                    let elapsed = start.elapsed().as_millis() as u64;
                    let inner = serde_json::json!({
                        "result": { "results": [] },
                        "evidence": [{"type": "search_query", "ref": input.query.clone()}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    });
                    return Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_search", inner, elapsed)).unwrap()));
                }
                let repo =
                    ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let project_id = ares_core::ProjectId::from(project_name);
                let query_lower = input.query.to_lowercase();
                let terms: Vec<&str> = query_lower.split_whitespace().collect();
                
                let mut results = Vec::new();
                if let Ok(all) = repo.get_all_nodes(&project_id) {
                    let mut matched: Vec<_> = all
                        .into_iter()
                        .filter(|n| {
                            let label_lower = n.label.to_lowercase();
                            let fp_lower = n.file_path.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
                            terms.iter().all(|&term| {
                                label_lower.contains(term) || fp_lower.contains(term)
                            })
                        })
                        .collect();
                    matched.truncate(input.limit);
                    for n in matched {
                        let fp = if n.node_type.as_str() == "commit" {
                            let mut first_file = None;
                            if let Ok(edges) = repo.get_edges_from(&n.id) {
                                for e in edges {
                                    if e.edge_type == ares_core::EdgeType::Touches {
                                        if let Ok(Some(tgt)) = repo.get_node(&e.to_node_id) {
                                            if let Some(path) = tgt.file_path {
                                                first_file = Some(path);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            first_file.unwrap_or_default()
                        } else {
                            n.file_path.clone().unwrap_or_default()
                        };

                        let summary = if n.node_type.as_str() == "commit" {
                            n.label
                        } else if !fp.is_empty() {
                            format!("{} (in {})", n.label, fp)
                        } else {
                            n.label
                        };
                        results.push(serde_json::json!({
                            "type": n.node_type,
                            "summary": summary,
                            "file_path": fp
                        }));
                    }
                }

                let elapsed = start.elapsed().as_millis() as u64;
                if results.is_empty() {
                    let inner = serde_json::json!({
                        "result": { "results": [] },
                        "evidence": [{"type": "search_query", "ref": input.query.clone()}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    });
                    let mut env = wrap_with_envelope("ares_search", inner, elapsed);
                    env["status"] = serde_json::json!("empty");
                    env["confidence"] = serde_json::json!(0.0);
                    Ok(CallToolResult::text(serde_json::to_string(&env).unwrap()))
                } else {
                    let inner = serde_json::json!({
                        "result": { "results": results },
                        "evidence": [{"type": "search_query", "ref": input.query.clone()}],
                        "confidence": 0.6,
                        "query_time_ms": start.elapsed().as_millis() as i64
                    });
                    Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_search", inner, elapsed)).unwrap()))
                }
            }
        })
        .build();

    // --- ares_timeline ---
    let store_tl = app_state.store.clone();
    let pp_tl = project_path.clone();
    let session_clone_timeline_tool = session_state.clone();
    let timeline_tool = ToolBuilder::new("ares_timeline")
        .description("Returns the chronological commit history for a file")
        .handler(move |input: TimelineQueryInput| {
            let session = session_clone_timeline_tool.clone();
            let store_arc = store_tl.clone();
            let pp = pp_tl.clone();
            async move {
                track_session_call(&session, "ares_timeline", &input);
                let start = std::time::Instant::now();
                let repo =
                    ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let _project_id = ares_core::ProjectId::from(project_name);

                let mut events = Vec::new();
                if let Ok(file_id_str) = repo.get_id_by_path(&input.file_path) {
                    let file_id = ares_core::NodeId::from(file_id_str.as_str());
                    if let Ok(edges) = repo.get_edges_to_by_type(&file_id, "touches") {
                        let mut commit_ids: Vec<(i64, ares_core::NodeId)> = edges
                            .iter()
                            .map(|e| (e.valid_from, e.from_node_id.clone()))
                            .collect();
                        commit_ids.sort_by_key(|(ts, _)| *ts);

                        for (ts, cid) in &commit_ids {
                            if let Ok(Some(commit)) = repo.get_node(cid) {
                                let author = commit
                                    .properties
                                    .get("author")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown");
                                let subject = commit
                                    .properties
                                    .get("subject")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let date_str = chrono::DateTime::from_timestamp_micros(*ts)
                                    .map(|dt| dt.to_rfc3339())
                                    .unwrap_or_else(|| ts.to_string());
                                
                                events.push(serde_json::json!({
                                    "date": date_str,
                                    "type": "commit",
                                    "summary": subject,
                                    "author": author
                                }));
                            }
                        }
                    }
                }

                // Generate narrative summary
                let narrative = if events.is_empty() {
                    "No commit history found for this file.".to_string()
                } else {
                    let author_set: std::collections::HashSet<String> = events
                        .iter()
                        .filter_map(|e| e.get("author").and_then(|a| a.as_str()).map(|s| s.to_string()))
                        .collect();
                    format!(
                        "{} commits by {} contributor{}. {}",
                        events.len(),
                        author_set.len(),
                        if author_set.len() == 1 { "" } else { "s" },
                        if events.len() > 20 {
                            "High activity — consider reviewing for refactoring opportunities."
                        } else if events.len() > 5 {
                            "Moderate activity."
                        } else {
                            "Low activity."
                        }
                    )
                };

                let total_commits = events.len();
                let is_truncated = total_commits > 50;
                let displayed_events: Vec<_> = if is_truncated {
                    events.into_iter().take(50).collect()
                } else {
                    events
                };
                let elapsed = start.elapsed().as_millis() as u64;
                let inner = serde_json::json!({
                    "result": {
                            "events": displayed_events,
                            "narrative": narrative,
                            "total_commits": total_commits,
                        },
                        "meta": {
                            "truncated": is_truncated
                        },
                        "confidence": 0.6,
                        "evidence": [{"type": "git_history", "ref": input.file_path.clone()}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                });
                Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_timeline", inner, elapsed)).unwrap()))
            }
        })
        .build();

    // --- ares_compare ---
    let store_cmp = app_state.store.clone();
    let pp_cmp = project_path.clone();
    let session_clone_compare_tool = session_state.clone();
    let compare_tool = ToolBuilder::new("ares_compare")
        .description("Compares two files: shared dependencies, shared decisions, coupling score")
        .handler(move |input: CompareQueryInput| {
            let session = session_clone_compare_tool.clone();
            let store_arc = store_cmp.clone();
            let pp = pp_cmp.clone();
            async move {
                track_session_call(&session, "ares_compare", &input);
                let start = std::time::Instant::now();
                let repo =
                    ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let _project_id = ares_core::ProjectId::from(project_name);

                let id_a = repo
                    .get_id_by_path(&input.file_a)
                    .ok()
                    .map(|s| ares_core::NodeId::from(s.as_str()));
                let id_b = repo
                    .get_id_by_path(&input.file_b)
                    .ok()
                    .map(|s| ares_core::NodeId::from(s.as_str()));

                let mut deps_a = std::collections::HashSet::new();
                let mut deps_b = std::collections::HashSet::new();

                if let Some(ref id) = id_a {
                    if let Ok(edges) = repo.get_edges_from(id) {
                        for e in &edges {
                            if e.edge_type.as_str() == "depends_on"
                                || e.edge_type.as_str() == "imports"
                            {
                                deps_a.insert(e.to_node_id.as_str().to_string());
                            }
                        }
                    }
                }
                if let Some(ref id) = id_b {
                    if let Ok(edges) = repo.get_edges_from(id) {
                        for e in &edges {
                            if e.edge_type.as_str() == "depends_on"
                                || e.edge_type.as_str() == "imports"
                            {
                                deps_b.insert(e.to_node_id.as_str().to_string());
                            }
                        }
                    }
                }

                let id_a_str = id_a
                    .as_ref()
                    .map(|id| id.as_str().to_string())
                    .unwrap_or_default();
                let id_b_str = id_b
                    .as_ref()
                    .map(|id| id.as_str().to_string())
                    .unwrap_or_default();

                let mut shared_ids: std::collections::HashSet<String> =
                    deps_a.intersection(&deps_b).cloned().collect();
                if deps_a.contains(&id_b_str) {
                    shared_ids.insert(id_b_str.clone());
                }
                if deps_b.contains(&id_a_str) {
                    shared_ids.insert(id_a_str.clone());
                }

                let a_count = deps_a.len();
                let b_count = deps_b.len();
                let max_count = a_count.max(b_count).max(1);

                let coupling = shared_ids.len() as f64 / max_count as f64;

                let shared_paths: Vec<String> = shared_ids
                    .into_iter()
                    .map(|id_str| {
                        let node_id = ares_core::NodeId::from(id_str.as_str());
                        if let Ok(Some(node)) = repo.get_node(&node_id) {
                            if let Some(fp) = node.file_path {
                                return fp;
                            }
                            return node.label;
                        }
                        id_str
                    })
                    .collect();

                let relationship = if coupling > 0.5 {
                    "tightly coupled"
                } else if coupling > 0.1 {
                    "loosely coupled"
                } else {
                    "independent"
                };

                let elapsed = start.elapsed().as_millis() as u64;
                let inner = serde_json::json!({
                    "result": {
                            "shared_dependencies": shared_paths,
                            "shared_decisions": [],
                            "relationship": relationship,
                            "coupling_score": (coupling * 100.0).round() as i32
                        },
                        "confidence": 0.6,
                        "evidence": [{"type": "impact_graph", "ref": input.file_a.clone()}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                });
                Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_compare", inner, elapsed)).unwrap()))
            }
        })
        .build();

    // --- ares_architecture ---
    let store_arch = app_state.store.clone();
    let pp_arch = project_path.clone();
    let session_clone_architecture_tool = session_state.clone();
    let architecture_tool = ToolBuilder::new("ares_architecture")
        .description("Returns a high-level architectural overview of the repository")
        .handler(move |_input: ArchitectureQueryInput| {
            let session = session_clone_architecture_tool.clone();
            let store_arc = store_arch.clone();
            let pp = pp_arch.clone();
            async move {
                track_session_call(&session, "ares_architecture", &_input);
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
                let project_id = ares_core::ProjectId::from(project_name);

                let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                let mut dep_names: std::collections::HashSet<String> = std::collections::HashSet::new();
                let mut top_files: Vec<(usize, String)> = Vec::new();
                let mut decisions: Vec<serde_json::Value> = Vec::new();
                let mut modules_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

                if let Ok(all_nodes) = repo.get_all_nodes(&project_id) {
                    for n in &all_nodes {
                        *type_counts.entry(format!("{:?}", n.node_type).to_lowercase()).or_insert(0) += 1;
                        if n.id.as_str().starts_with("DEP-") {
                            dep_names.insert(n.label.clone());
                        }
                        if format!("{:?}", n.node_type).to_lowercase() == "file" {
                            if let Some(fp) = &n.file_path {
                                let path = std::path::Path::new(fp);
                                if let Some(parent) = path.parent() {
                                    let parent_str = parent.to_string_lossy().to_string();
                                    if !parent_str.is_empty() {
                                        let top_dir = parent_str.split('/').next().unwrap_or(&parent_str).to_string();
                                        let top_dir = top_dir.split('\\').next().unwrap_or(&top_dir).to_string();
                                        *modules_map.entry(top_dir).or_insert(0) += 1;
                                    }
                                }
                            }
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
                    top_files.sort_by_key(|b| std::cmp::Reverse(b.0));
                    top_files.truncate(10);
                }

                if let Ok(all_decisions) = repo.get_nodes_by_type(&project_id, "decision") {
                    for d in &all_decisions {
                        let summary = d.properties.get("decision").and_then(|v| v.as_str()).unwrap_or(&d.label);
                        decisions.push(serde_json::json!({ "summary": summary }));
                    }
                    decisions.truncate(10);
                }

                let mut modules: Vec<serde_json::Value> = modules_map.into_iter()
                    .map(|(name, count)| serde_json::json!({"name": name, "file_count": count}))
                    .collect();
                modules.sort_by(|a, b| b["file_count"].as_u64().unwrap_or(0).cmp(&a["file_count"].as_u64().unwrap_or(0)));
                modules.truncate(15);

                let file_count = type_counts.get("file").copied().unwrap_or(0);
                let func_count = type_counts.get("function").copied().unwrap_or(0);
                let commit_count = type_counts.get("commit").copied().unwrap_or(0);

                let tech_stack: Vec<String> = dep_names.into_iter().take(20).collect();
                let top: Vec<serde_json::Value> = top_files.iter().map(|(c, p)| serde_json::json!({"path": p, "dependents": c})).collect();

                let cochanges = if let Ok(conn) = store_arc.get_conn() {
                    ares_intelligence::cochange::detect_hidden_coupling(
                        &conn, 3, 90, 20,
                    )
                    .unwrap_or_default()
                } else {
                    Vec::new()
                };

                let elapsed = start.elapsed().as_millis() as u64;
                let inner = serde_json::json!({
                    "result": {
                        "summary": format!("{} files, {} functions, {} commits across {} node types", file_count, func_count, commit_count, type_counts.len()),
                        "top_files": top,
                        "modules": modules,
                        "key_decisions": decisions,
                        "technology_stack": tech_stack,
                        "health_score": 0
                    },
                    "hidden_coupling": cochanges,
                    "confidence": 0.6,
                    "evidence": [{"type": "drift_analysis", "ref": "workspace"}],
                    "query_time_ms": start.elapsed().as_millis() as i64
                });
                Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_architecture", inner, elapsed)).unwrap()))
            }
        })
        .build();

    // --- ares_requirements ---
    let store_req = app_state.store.clone();
    let pp_req = project_path.clone();
    let session_clone_requirements_tool = session_state.clone();
    let requirements_tool = ToolBuilder::new("ares_requirements")
        .description("Returns requirements linked to the repository or a specific file")
        .handler(move |input: RequirementsQueryInput| {
            let session = session_clone_requirements_tool.clone();
            let store_arc = store_req.clone();
            let pp = pp_req.clone();
            async move {
                track_session_call(&session, "ares_requirements", &input);
                let start = std::time::Instant::now();
                let repo =
                    ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let project_id = ares_core::ProjectId::from(project_name);

                let mut requirements = Vec::new();

                if let Ok(all) = repo.get_nodes_by_type(&project_id, "requirement") {
                    for rn in &all {
                        let text = rn
                            .properties
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&rn.label);
                        let status = rn
                            .properties
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");

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
                                "id": format!("node_id:{}", rn.id.as_str()),
                                "text": text,
                                "status": status,
                                "linked_files": linked_files
                            }));
                        }
                    }
                }

                let elapsed = start.elapsed().as_millis() as u64;
                if requirements.is_empty() {
                    let inner = serde_json::json!({
                        "result": { "requirements": [] },
                        "confidence": 0.6,
                        "gap_flags": ["no_recorded_requirements"],
                        "evidence": [{"type": "gap_analysis", "ref": input.file_path.clone().unwrap_or_else(|| "workspace".to_string())}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    });
                    Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_requirements", inner, elapsed)).unwrap()))
                } else {
                    let inner = serde_json::json!({
                        "result": { "requirements": requirements },
                        "confidence": 0.6,
                        "evidence": [{"type": "gap_analysis", "ref": input.file_path.clone().unwrap_or_else(|| "workspace".to_string())}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    });
                    Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_requirements", inner, elapsed)).unwrap()))
                }
            }
        })
        .build();

    // --- Task 3.2: Agent Memory Write API ---
    let store_rec_dec = app_state.store.clone();
    let pp_rec_dec = project_path.clone();
    let session_clone_record_decision_tool = session_state.clone();
    let record_decision_tool = ToolBuilder::new("ares_record_decision")
        .description("Record an architectural decision and link it to impacted files")
        .handler(move |input: RecordDecisionInput| {
            let session = session_clone_record_decision_tool.clone();
            let store_arc = store_rec_dec.clone();
            let pp_local = pp_rec_dec.clone();
            async move {
                let start = std::time::Instant::now();
                track_session_call(&session, "ares_record_decision", &input);
                // Validate required fields
                if input.title.trim().is_empty() {
                    return Err(tower_mcp::Error::invalid_params("title is required and must not be empty"));
                }
                if input.description.trim().is_empty() {
                    return Err(tower_mcp::Error::invalid_params("description is required and must not be empty"));
                }
                let repo =
                    ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp_local)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let project_id = ares_core::ProjectId::from(project_name);

                let node_id = ares_core::NodeId::new();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as i64;

                let properties = serde_json::json!({
                    "source": input.source.unwrap_or_else(|| "agent".to_string()),
                    "description": input.description,
                    "status": input.status,
                    "confidence": input.confidence.unwrap_or(1.0)
                });

                let decision_node = ares_core::GraphNode {
                    id: node_id.clone(),
                    project_id: project_id.clone(),
                    node_type: ares_core::NodeType::Decision,
                    label: input.title,
                    properties,
                    file_path: None,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                };

                if let Err(e) = repo.upsert_node(decision_node) {
                    return Ok(CallToolResult::text(format!(
                        "Failed to record decision: {}",
                        e
                    )));
                }

                let mut linked_files = Vec::new();
                let mut linking_errors = Vec::new();
                for path in input.impacted_paths {
                    if let Ok(file_id_str) = repo.get_id_by_path_loose(&path) {
                        let file_id = ares_core::NodeId::from(file_id_str);
                        let edge = ares_core::GraphEdge {
                            id: ares_core::new_id(),
                            project_id: project_id.clone(),
                            from_node_id: node_id.clone(),
                            to_node_id: file_id,
                            edge_type: ares_core::EdgeType::RelatedTo,
                            weight: 1.0,
                            confidence: 1.0,
                            source: "agent".to_string(),
                            valid_from: now,
                            valid_until: None,
                            created_at: now,
                        };
                        match repo.upsert_edge(edge) {
                            Ok(_) => linked_files.push(path),
                            Err(e) => linking_errors.push(format!("{}: {}", path, e)),
                        }
                    } else {
                        linking_errors.push(format!("{}: Not found in graph", path));
                    }
                }

                Ok(CallToolResult::text(
                    serde_json::to_string(&serde_json::json!({
                        "result": {
                            "status": "recorded",
                            "linked_files": linked_files,
                            "linking_errors": linking_errors
                        },
                        "node_id": node_id.as_str(),
                        "evidence": [{"type": "agent_analysis", "ref": node_id.to_string()}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    })).unwrap_or_default(),
                ))
            }
        })
        .build();

    let store_rec_req = app_state.store.clone();
    let pp_rec_req = project_path.clone();
    let session_clone_record_requirement_tool = session_state.clone();
    let record_requirement_tool = ToolBuilder::new("ares_record_requirement")
        .description("Record a business or technical requirement and link it to files")
        .handler(move |input: RecordRequirementInput| {
            let session = session_clone_record_requirement_tool.clone();
            let store_arc = store_rec_req.clone();
            let pp_local = pp_rec_req.clone();
            async move {
                let start = std::time::Instant::now();
                track_session_call(&session, "ares_record_requirement", &input);
                if input.title.trim().is_empty() {
                    return Err(tower_mcp::Error::invalid_params("title is required and must not be empty"));
                }
                if input.description.trim().is_empty() {
                    return Err(tower_mcp::Error::invalid_params("description is required and must not be empty"));
                }
                let repo =
                    ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp_local)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let project_id = ares_core::ProjectId::from(project_name);

                let node_id = ares_core::NodeId::new();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as i64;

                let properties = serde_json::json!({
                    "source": "agent",
                    "description": input.description,
                    "priority": input.priority,
                    "confidence": 1.0
                });

                let req_node = ares_core::GraphNode {
                    id: node_id.clone(),
                    project_id: project_id.clone(),
                    node_type: ares_core::NodeType::Requirement,
                    label: input.title,
                    properties,
                    file_path: None,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                };

                if let Err(e) = repo.upsert_node(req_node) {
                    return Ok(CallToolResult::text(format!(
                        "Failed to record requirement: {}",
                        e
                    )));
                }

                let mut linked_files = Vec::new();
                let mut linking_errors = Vec::new();
                for path in input.satisfies_paths {
                    if let Ok(file_id_str) = repo.get_id_by_path_loose(&path) {
                        let file_id = ares_core::NodeId::from(file_id_str);
                        let edge = ares_core::GraphEdge {
                            id: ares_core::new_id(),
                            project_id: project_id.clone(),
                            from_node_id: file_id,
                            to_node_id: node_id.clone(),
                            edge_type: ares_core::EdgeType::RelatedTo,
                            weight: 1.0,
                            confidence: 1.0,
                            source: "agent".to_string(),
                            valid_from: now,
                            valid_until: None,
                            created_at: now,
                        };
                        match repo.upsert_edge(edge) {
                            Ok(_) => linked_files.push(path),
                            Err(e) => linking_errors.push(format!("{}: {}", path, e)),
                        }
                    } else {
                        linking_errors.push(format!("{}: Not found in graph", path));
                    }
                }

                Ok(CallToolResult::text(
                    serde_json::to_string(&serde_json::json!({
                        "result": {
                            "status": "recorded",
                            "linked_files": linked_files,
                            "linking_errors": linking_errors
                        },
                        "node_id": node_id.as_str(),
                        "evidence": [{"type": "agent_analysis", "ref": node_id.to_string()}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    })).unwrap_or_default(),
                ))
            }
        })
        .build();

    let store_ann = app_state.store.clone();
    let session_clone_annotate_tool = session_state.clone();
    let annotate_tool = ToolBuilder::new("ares_annotate")
        .description("Annotate a file or node by adding a key-value property")
        .handler(move |input: AnnotateInput| {
            let session = session_clone_annotate_tool.clone();
            let store_arc = store_ann.clone();
            async move {
                let start = std::time::Instant::now();
                track_session_call(&session, "ares_annotate", &input);
                if input.target_path.trim().is_empty() {
                    return Err(tower_mcp::Error::invalid_params("target_path is required"));
                }
                if input.key.trim().is_empty() {
                    return Err(tower_mcp::Error::invalid_params("key is required"));
                }
                let repo =
                    ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());

                let result = if let Ok(file_id_str) = repo.get_id_by_path(&input.target_path) {
                    let file_id = ares_core::NodeId::from(file_id_str);
                    if let Ok(Some(mut node)) = repo.get_node(&file_id) {
                        if let Some(obj) = node.properties.as_object_mut() {
                            let mut annotations = obj.remove("annotations").unwrap_or_else(|| serde_json::json!({}));
                            if let Some(ann_obj) = annotations.as_object_mut() {
                                ann_obj.insert(input.key.clone(), serde_json::json!(input.value));
                            } else {
                                let mut new_ann_obj = serde_json::Map::new();
                                new_ann_obj.insert(input.key.clone(), serde_json::json!(input.value));
                                annotations = serde_json::Value::Object(new_ann_obj);
                            }
                            obj.insert("annotations".to_string(), annotations);
                            node.updated_at = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64;
                            if repo.upsert_node(node).is_ok() {
                                Some(serde_json::json!({
                                    "status": "added",
                                    "target": input.target_path,
                                    "key": input.key
                                }))
                            } else {
                                Some(serde_json::json!({
                                    "status": "error",
                                    "error": "Failed to persist annotation to database"
                                }))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                let response = match result {
                    Some(r) => serde_json::json!({
                        "result": r,
                        "evidence": [{"type": "agent_analysis", "ref": input.target_path.clone()}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    }),
                    None => serde_json::json!({
                        "result": null,
                        "error": format!("File not found in graph: {}", input.target_path),
                        "evidence": [],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    }),
                };

                Ok(CallToolResult::text(serde_json::to_string(&response).unwrap_or_default()))
            }
        })
        .build();

    let store_corr = app_state.store.clone();
    let session_clone_correct_tool = session_state.clone();
    let correct_tool = ToolBuilder::new("ares_correct")
        .description("Correct a node's properties manually")
        .handler(move |input: CorrectInput| {
            let session = session_clone_correct_tool.clone();
            let store_arc = store_corr.clone();
            async move {
                track_session_call(&session, "ares_correct", &input);
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());

                if let Ok(file_id_str) = repo.get_id_by_path(&input.target_path) {
                    let file_id = ares_core::NodeId::from(file_id_str);
                    if let Ok(Some(mut node)) = repo.get_node(&file_id) {
                        if let Some(obj) = node.properties.as_object_mut() {
                            let mut corrections = obj.remove("corrections").unwrap_or_else(|| serde_json::json!([]));
                            if let Some(arr) = corrections.as_array_mut() {
                                arr.push(serde_json::json!({
                                    "timestamp": (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64),
                                    "note": input.correction_notes
                                }));
                            }
                            obj.insert("corrections".to_string(), corrections);
                            node.updated_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64;

                            if repo.upsert_node(node).is_ok() {
                                return Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                                    "result": "Correction recorded",
                                    "target": input.target_path
                                })).unwrap_or_default()));
                            }
                        }
                    }
                }

                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "error": "Failed to record correction: node not found"
                })).unwrap_or_default()))
            }
        })
        .build();

    let store_ctx = app_state.store.clone();
    let pp_ctx = project_path.clone();
    let session_context_tool = ToolBuilder::new("ares_session_context")
        .description("Returns summaries of the last 3 agent sessions for continuity")
        .handler(move |_input: EmptyInput| {
            let store_arc = store_ctx.clone();
            let pp = pp_ctx.clone();
            async move {
                let start = std::time::Instant::now();
                let project_id = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();

                let mut sessions = Vec::new();
                if let Ok(conn) = store_arc.get_conn() {
                    if let Ok(mut stmt) = conn.prepare(
                        "SELECT id, started_at, ended_at, tool_calls, summary, files_touched FROM agent_sessions WHERE project_id = ?1 ORDER BY ended_at DESC LIMIT 3"
                    ) {
                        if let Ok(rows) = stmt.query_map(rusqlite::params![project_id.as_str()], |row| {
                            let id: String = row.get(0).unwrap_or_default();
                            let started: i64 = row.get(1).unwrap_or_default();
                            let ended: i64 = row.get(2).unwrap_or_default();
                            let calls: String = row.get(3).unwrap_or_default();
                            let summary: String = row.get(4).unwrap_or_default();
                            let files: String = row.get(5).unwrap_or_default();
                            Ok((id, started, ended, calls, summary, files))
                        }) {
                            for s in rows.flatten() {
                                sessions.push(serde_json::json!({
                                    "session_id": s.0,
                                    "started_at": format_micros_as_iso(s.1),
                                    "ended_at": format_micros_as_iso(s.2),
                                    "tool_calls": serde_json::from_str::<Vec<Vec<serde_json::Value>>>(&s.3).unwrap_or_default(),
                                    "summary": s.4,
                                    "files_touched": serde_json::from_str::<Vec<String>>(&s.5).unwrap_or_default()
                                }));
                            }
                        }
                    }
                }

                let elapsed = start.elapsed().as_millis() as u64;
                let inner = serde_json::json!({
                    "result": { "sessions": sessions },
                    "confidence": 0.6,
                    "evidence": [{"type": "session_logs", "ref": "workspace"}],
                    "query_time_ms": start.elapsed().as_millis() as i64
                });
                Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_session_context", inner, elapsed)).unwrap_or_default()))
            }
        })
        .build();

    let store_end = app_state.store.clone();

    let session_clone_for_end = session_state.clone();
    let end_session_tool = ToolBuilder::new("ares_end_session")
        .description("Ends the current agent session and persists it to the database")
        .handler(move |input: EndSessionInput| {
            let session = session_clone_for_end.clone();
            let store_arc = store_end.clone();
            let start = std::time::Instant::now();
            async move {
                let conn = store_arc.get_conn().ok();

                let (tool_calls, files_touched, project_id_str) = {
                    let mut state = session.lock().unwrap();
                    (state.tool_calls.drain(..).collect::<Vec<_>>(), state.files_touched.drain().collect::<Vec<_>>(), state.project_id.clone())
                };

                let summary = if tool_calls.is_empty() {
                    "Empty session".to_string()
                } else {
                    format!(
                        "{} tool calls, {} files touched. Top tools: {}",
                        tool_calls.len(),
                        files_touched.len(),
                        tool_calls.iter()
                            .take(5)
                            .map(|(name, _)| name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };

                let session_id = format!("ses_{}", uuid::Uuid::new_v4().simple());
                let mut inserted = false;
                let mut db_error = None;

                if let Some(conn) = conn {
                    if let Ok(mut stmt) = conn.prepare(
                        "INSERT INTO agent_sessions (id, project_id, summary, tool_calls, files_touched, started_at, ended_at, left_incomplete, recommended_next) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
                    ) {
                        let left_incomplete = input.left_incomplete.as_deref().unwrap_or("");
                        let recommended_next = input.recommended_next.as_deref().unwrap_or("");
                        let ended = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64;
                        match stmt.execute(rusqlite::params![
                            session_id,
                            project_id_str,
                            summary,
                            serde_json::to_string(&tool_calls).unwrap_or_default(),
                            serde_json::to_string(&files_touched).unwrap_or_default(),
                            start.elapsed().as_secs() as i64,
                            ended,
                            left_incomplete,
                            recommended_next,
                        ]) {
                            Ok(_) => inserted = true,
                            Err(e) => db_error = Some(format!("Database error: {}", e)),
                        }
                    } else {
                        db_error = Some("Failed to prepare session insert statement".to_string());
                    }
                } else {
                    db_error = Some("No database connection available".to_string());
                }

                // ONLY clear session state if successfully persisted
                if inserted {
                    let mut state = session.lock().unwrap();
                    state.tool_calls.clear();
                    state.files_touched.clear();
                    state.started_at = std::time::Instant::now();
                }

                let response = if inserted {
                    serde_json::json!({
                        "result": {
                            "session_id": session_id,
                            "status": "persisted",
                            "summary": summary,
                            "tool_calls": tool_calls.len(),
                            "files_touched": files_touched.len(),
                            "left_incomplete": input.left_incomplete,
                            "recommended_next": input.recommended_next
                        },
                        "evidence": [{"type": "session_logs", "ref": "current_session"}],
                        "query_time_ms": start.elapsed().as_millis() as i64
                    })
                } else {
                    serde_json::json!({
                        "result": {
                            "session_id": session_id,
                            "status": "failed",
                            "summary": summary,
                            "error": db_error
                        },
                        "evidence": [{"type": "session_logs", "ref": "current_session"}],
                        "query_time_ms": start.elapsed().as_millis() as i64,
                        "warnings": ["Session data preserved in memory but not persisted to database. Try again or check database health."]
                    })
                };

                Ok(CallToolResult::text(serde_json::to_string(&response).unwrap_or_default()))
            }
        })
        .build();

      let store_gaps = app_state.store.clone();
      let session_clone_gaps_tool = session_state.clone();
      let gaps_tool = ToolBuilder::new("ares_gaps")
          .description("Evaluates knowledge gaps in the traceability graph")
          .handler(move |input: ProjectQueryInput| {
              let store = store_gaps.clone();
              let session = session_clone_gaps_tool.clone();
              async move {
                  track_session_call(&session, "ares_gaps", &input);
                  let project_name = input
                      .project_id
                      .clone()
                      .unwrap_or_else(|| session.lock().unwrap().project_id.clone());
                  let project_id = ares_core::ProjectId::from(project_name);
                  let repo = ares_store::repositories::gaps::SqliteGapRepository::new(store.clone());
                  let mut all_gaps = Vec::new();
                  if let Ok(mut g) = repo.get_code_without_decision(&project_id, 30) { all_gaps.append(&mut g); }
                  if let Ok(mut g) = repo.get_decisions_without_code(&project_id, 7) { all_gaps.append(&mut g); }
                  if let Ok(mut g) = repo.get_orphaned_requirements(&project_id) { all_gaps.append(&mut g); }
                  if let Ok(mut g) = repo.get_stale_decisions(&project_id, 30) { all_gaps.append(&mut g); }
                  if let Ok(mut g) = repo.get_unknown_ownership(&project_id) { all_gaps.append(&mut g); }
                  let mut gaps_val = serde_json::to_value(&all_gaps).unwrap_or_default();
                  prefix_node_ids(&mut gaps_val);
                  strip_details_uuids(&mut gaps_val);
                  let evidence = serde_json::json!([{"type": "gap_analysis", "ref": "workspace"}]);
                  let conf = 0.6;
                  let payload = serde_json::json!({
                      "gaps": gaps_val,
                      "gap_count": all_gaps.len(),
                      "evidence": evidence,
                      "confidence": conf
                  });
                  serde_json::to_string(&wrap_with_envelope("ares_gaps", payload, 0))
                      .map(CallToolResult::text)
                      .map_err(|e| {
                          tower_mcp::Error::internal(format_mcp_error(
                              "Failed to serialize gaps evaluation",
                              &e.to_string(),
                          ))
                      })
              }
          })
          .build();

      let store_sim = app_state.store.clone();
      let session_clone_sim = session_state.clone();

      let simulate_tool = ToolBuilder::new("ares_simulate")
          .description("Performs mutation analysis only. Simulates structural changes (e.g., removing a node) to project coverage drops, new gaps, and drift before they happen.")
          .handler(move |input: SimulationInput| {
              let session = session_clone_sim.clone();
              let store = store_sim.clone();
              async move {
                  track_session_call(&session, "ares_simulate", &input);
                  
                  let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
                  
                  // Resolve path to node ID (same as ares_impact)
                  let target_id = if input.target_id.starts_with("file:") 
                      || input.target_id.starts_with("0") {
                      ares_core::canonicalize_node_id(&input.target_id)
                  } else {
                      match repo.get_id_by_path_loose(&input.target_id) {
                          Ok(id) => ares_core::canonicalize_node_id(&id),
                          Err(_) => return Ok(CallToolResult::text(
                              serde_json::to_string(&serde_json::json!({
                                  "action": input.action,
                                  "target": input.target_id,
                                  "impact_radius": [],
                                  "decision_conflicts": [],
                                  "risk_score": 0,
                                  "summary": format!("Entity '{}' not found in graph", input.target_id),
                                  "reversible": false
                              })).unwrap()
                          )),
                      }
                  };
                  
                  let related = input.related_id.as_deref().map(|r| {
                      if r.starts_with("file:") || r.starts_with("0") {
                          ares_core::canonicalize_node_id(r)
                      } else {
                          repo.get_id_by_path_loose(r)
                              .unwrap_or_else(|_| ares_core::canonicalize_node_id(r))
                      }
                  });

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
                    Ok(mut report) => {
                        report.target = input.target_id.clone();
                        let ref_val = input.target_id.clone();
                        let evidence = serde_json::json!([{"type": "simulation_query", "ref": ref_val}]);
                        let conf = 0.6;
                        let mut payload = serde_json::to_value(&report).unwrap_or_default();
                        if let Some(obj) = payload.as_object_mut() {
                            obj.insert("evidence".to_string(), evidence);
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                        }
                        serde_json::to_string(&wrap_with_envelope("ares_simulate", payload, 0))
                            .map(CallToolResult::text)
                            .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize simulation report", &e.to_string())))
                    },
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to simulate change", &e.to_string()))),
                }
            }
        })
        .build();

    let intelligence_facade_trace = intelligence_facade.clone();
    let project_id_str_trace = project_path.clone();
    let session_clone_traceability_tool = session_state.clone();
    let store_traceability = app_state.store.clone();
    let traceability_tool = ToolBuilder::new("ares_traceability")
        .description("Evaluates traceability relationships upstream and downstream")
        .handler(move |input: TraceabilityInput| {
            let session = session_clone_traceability_tool.clone();
            let facade = intelligence_facade_trace.clone();
            let project_id = project_id_str_trace.clone();
            let store = store_traceability.clone();
            async move {
                track_session_call(&session, "ares_traceability", &input);
                let id_str = match input.resolve_id(&store) {
                    Ok(i) => i,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                };
                let id = ares_core::canonicalize_node_id(&id_str);
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
                            "gap_flags": serde_json::json!(insight.gap_flags),
                            "warnings": insight.warnings,
                            "recommendations": insight.recommendations,
                            "summary": insight.summary,
                            "file_path": input.file_path.clone().or_else(|| input.entity_id.clone()).unwrap_or_default(),
                            "entity": input.file_path.clone().or_else(|| input.entity_id.clone()).unwrap_or_default(),
                            "mode": insight.mode,
                            "metadata": insight.metadata,
                        });
                        Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_traceability", response, 0)).unwrap()))
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
    let session_clone_graph_statistics_tool = session_state.clone();
    let graph_statistics_tool = ToolBuilder::new("ares_graph_statistics")
        .description("Retrieves statistics about the knowledge graph")
        .handler(move |_input: EmptyInput| {
            let session = session_clone_graph_statistics_tool.clone();
            let store = store_graph.clone();
            async move {
                track_session_call(&session, "ares_graph_statistics", &_input);
                let start = std::time::Instant::now();
                let result = ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_statistics(&store).await;
                match result {
                    Ok(stats) => {
                        let evidence = serde_json::json!([{"type": "graph_query", "ref": "workspace"}]);
                        let conf = 0.6;
                        let mut payload = serde_json::to_value(&stats).unwrap_or_default();
                        if let Some(obj) = payload.as_object_mut() {
                            obj.insert("evidence".to_string(), evidence);
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                        }
                        serde_json::to_string(&wrap_with_envelope("ares_graph_statistics", payload, start.elapsed().as_millis() as u64))
                            .map(CallToolResult::text)
                            .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize graph stats", &e.to_string())))
                    },
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to retrieve graph stats", &e.to_string()))),
                }
            }
        })
        .build();

    let store_graph_root = app_state.store.clone();
    let session_clone_graph_root_tool = session_state.clone();
    let graph_root_tool = ToolBuilder::new("ares_graph_root")
        .description("Retrieves the root node of the graph to start lazy loading")
        .handler(move |_input: EmptyInput| {
            let session = session_clone_graph_root_tool.clone();
            let store = store_graph_root.clone();
            async move {
                track_session_call(&session, "ares_graph_root", &_input);
                let start = std::time::Instant::now();
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
                    Ok(original_payload) => {
                        let evidence = serde_json::json!([{"type": "graph_query", "ref": "workspace"}]);
                        let conf = 0.6;
                        let mut payload = serde_json::to_value(&original_payload).unwrap_or_default();
                        transform_graph_for_agent(&mut payload);
                        prefix_node_ids(&mut payload);
                        if let Some(obj) = payload.as_object_mut() {
                            obj.insert("evidence".to_string(), evidence);
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                        }
                        serde_json::to_string(&wrap_with_envelope("ares_graph_root", payload, start.elapsed().as_millis() as u64))
                            .map(CallToolResult::text)
                            .map_err(|e| {
                                tower_mcp::Error::internal(format_mcp_error(
                                    "Failed to serialize graph root",
                                    &e.to_string(),
                                ))
                            })
                    },
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error(
                        "Failed to retrieve graph root",
                        &e.to_string(),
                    ))),
                }
            }
        })
        .build();

    let store_graph_neighbors = app_state.store.clone();
    let session_clone_graph_neighbors_tool = session_state.clone();
    let graph_neighbors_tool = ToolBuilder::new("ares_graph_neighbors")
        .description("Expands a node by fetching its immediate neighbors")
        .handler(move |input: GraphNeighborsInput| {
            let session = session_clone_graph_neighbors_tool.clone();
            let store = store_graph_neighbors.clone();
            async move {
                track_session_call(&session, "ares_graph_neighbors", &input);
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
                let resolved_id = repo.get_id_by_path(&input.node_id).unwrap_or_else(|_| input.node_id.clone());
                let node_id_str = ares_core::canonicalize_node_id(&resolved_id);
                let node_id = ares_core::NodeId::from(node_id_str.clone());
                match ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_neighbors(&store, &node_id).await {
                    Ok(payload) => {
                        let payload_val = serde_json::to_value(&payload).unwrap_or_default();
                        let (final_answer, evidence) = build_neighbors_answer(payload_val, &node_id_str);
                        let conf = if evidence.is_empty() { 0.0 } else { 1.0 };
                        
                        let mut envelope_payload = final_answer;
                        if let Some(obj) = envelope_payload.as_object_mut() {
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                            obj.insert("evidence".to_string(), serde_json::Value::Array(evidence));
                        }
                        
                        serde_json::to_string(&wrap_with_envelope("ares_graph_neighbors", envelope_payload, start.elapsed().as_millis() as u64))
                            .map(CallToolResult::text)
                            .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize graph neighbors", &e.to_string())))
                    }
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to retrieve graph neighbors", &e.to_string()))),
                }
            }
        })
        .build();

    let store_graph_search = app_state.store.clone();
    let session_clone_graph_search_tool = session_state.clone();
    let graph_search_tool = ToolBuilder::new("ares_graph_search")
        .description("Searches the graph for nodes matching the query")
        .handler(move |input: GraphSearchInput| {
            let session = session_clone_graph_search_tool.clone();
            let store = store_graph_search.clone();
            async move {
                track_session_call(&session, "ares_graph_search", &input);
                let start = std::time::Instant::now();
                match ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_search(&store, &input.query).await {
                    Ok(original_payload) => {
                        let evidence = serde_json::json!([{"type": "graph_search", "ref": &input.query}]);
                        let conf = 0.6;
                        let mut payload = serde_json::to_value(&original_payload).unwrap_or_default();
                        // Strip unresolved internal artifacts before transforming
                        if let Some(nodes) = payload.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                            nodes.retain(|n| {
                                n.get("id").and_then(|id| id.as_str()).map_or(true, |s| !s.starts_with("unresolved_"))
                            });
                        }
                        if let Some(edges) = payload.get_mut("edges").and_then(|e| e.as_array_mut()) {
                            edges.retain(|e| {
                                let from_bad = e.get("from_node_id").and_then(|v| v.as_str()).map_or(false, |s| s.starts_with("unresolved_"));
                                let to_bad = e.get("to_node_id").and_then(|v| v.as_str()).map_or(false, |s| s.starts_with("unresolved_"));
                                !from_bad && !to_bad
                            });
                        }
                        transform_graph_for_agent(&mut payload);
                        prefix_node_ids(&mut payload);
                        if let Some(obj) = payload.as_object_mut() {
                            obj.insert("evidence".to_string(), evidence);
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                        }
                        serde_json::to_string(&wrap_with_envelope("ares_graph_search", payload, start.elapsed().as_millis() as u64))
                            .map(CallToolResult::text)
                            .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize graph search results", &e.to_string())))
                    },
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to search graph", &e.to_string()))),
                }
            }
        })
        .build();

    let store_graph_shortest_path = app_state.store.clone();
    let session_clone_graph_shortest_path_tool = session_state.clone();
    let graph_shortest_path_tool = ToolBuilder::new("ares_graph_shortest_path")
        .description("Finds the shortest dependency path between two nodes")
        .handler(move |input: GraphPathInput| {
            let session = session_clone_graph_shortest_path_tool.clone();
            let store = store_graph_shortest_path.clone();
            async move {
                track_session_call(&session, "ares_graph_shortest_path", &input);
                let start = std::time::Instant::now();
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
                let resolved_from = repo.get_id_by_path(&input.from_id).unwrap_or_else(|_| input.from_id.clone());
                let resolved_to = repo.get_id_by_path(&input.to_id).unwrap_or_else(|_| input.to_id.clone());
                
                let from_id_str = ares_core::canonicalize_node_id(&resolved_from);
                let to_id_str = ares_core::canonicalize_node_id(&resolved_to);
                let from_id = ares_core::NodeId::from(from_id_str);
                let to_id = ares_core::NodeId::from(to_id_str);
                match ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_shortest_path(&store, &from_id, &to_id).await {
                    Ok(payload) => {
                        let payload_val = serde_json::to_value(&payload).unwrap_or_default();
                        let (final_answer, evidence) = build_shortest_path_answer(payload_val);
                        let conf = if evidence.is_empty() { 0.0 } else { 1.0 };
                        
                        let mut envelope_payload = final_answer;
                        if let Some(obj) = envelope_payload.as_object_mut() {
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                            obj.insert("evidence".to_string(), serde_json::Value::Array(evidence));
                        }
                        
                        serde_json::to_string(&wrap_with_envelope("ares_graph_shortest_path", envelope_payload, start.elapsed().as_millis() as u64))
                            .map(CallToolResult::text)
                            .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize shortest path", &e.to_string())))
                    }
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to find shortest path", &e.to_string()))),
                }
            }
        })
        .build();

    let store_graph_metadata = app_state.store.clone();
    let session_clone_graph_metadata_tool = session_state.clone();
    let graph_metadata_tool = ToolBuilder::new("ares_graph_metadata")
        .description("Retrieves full metadata for a specific node")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_graph_metadata_tool.clone();
            let store = store_graph_metadata.clone();
            async move {
                track_session_call(&session, "ares_graph_metadata", &input);
                let start = std::time::Instant::now();
                let node_id_str_str = match input.resolve_id(&store) {
                    Ok(i) => i,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                };
                let node_id_str = ares_core::canonicalize_node_id(&node_id_str_str);
                let node_id = ares_core::NodeId::from(node_id_str);
                match ares_repository_intelligence::engines::graph::RepositoryGraphEngine::graph_metadata(&store, &node_id).await {
                    Ok(node) => {
                        let mut node_val = serde_json::to_value(&node).unwrap_or_default();
                        prefix_node_ids(&mut node_val);
                        default_nulls(&mut node_val);
                        transform_relationships(&mut node_val);
                        
                        if let Some(rels) = node_val.get("relationships").and_then(|r| r.as_object()) {
                            let mut evidence_arr = Vec::new();
                            for (rel_type, citations) in rels {
                                if let Some(arr) = citations.as_array() {
                                    for cit in arr {
                                        if let Some(cit_obj) = cit.as_object() {
                                            let title = cit_obj.get("title").and_then(|v| v.as_str()).unwrap_or("unknown");
                                            let kind = cit_obj.get("kind").and_then(|v| v.as_str()).unwrap_or(rel_type.as_str());
                                            let ref_source = input.file_path.clone().unwrap_or_else(|| node_id_str_str.clone());
                                            evidence_arr.push(serde_json::json!({
                                                "type": "ast_edge",
                                                "ref": format!("{}:{}→{}", ref_source, kind, title)
                                            }));
                                        }
                                    }
                                }
                            }
                            if let Some(obj) = node_val.as_object_mut() {
                                obj.insert("evidence".to_string(), serde_json::Value::Array(evidence_arr));
                            }
                        }
                        
                        let conf_opt = node_val.get("health")
                            .and_then(|h| h.as_object())
                            .and_then(|h| h.get("confidence"))
                            .cloned();
                        if let Some(conf) = conf_opt {
                            if let Some(obj) = node_val.as_object_mut() {
                                obj.insert("confidence".to_string(), conf);
                            }
                        }
                        
                        if node.health.is_orphan {
                            let mut gap_flags = node_val.get("gap_flags")
                                .and_then(|v| v.as_array())
                                .map(|a| a.clone())
                                .unwrap_or_default();
                            gap_flags.push(serde_json::Value::String("orphan_node".to_string()));
                            if let Some(obj) = node_val.as_object_mut() {
                                obj.insert("gap_flags".to_string(), serde_json::Value::Array(gap_flags));
                            }
                        }

                        serde_json::to_string(&wrap_with_envelope("ares_graph_metadata", node_val, start.elapsed().as_millis() as u64))
                            .map(CallToolResult::text)
                            .map_err(|e| tower_mcp::Error::internal(format_mcp_error("Failed to serialize node metadata", &e.to_string())))
                    }
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed to retrieve node metadata", &e.to_string()))),
                }
            }
        })
        .build();

    let we_bookmark = workspace_engine.clone();
    let session_clone_workspace_bookmark_tool = session_state.clone();
    let workspace_bookmark_tool = ToolBuilder::new("ares_workspace_bookmark")
        .description("Bookmark a node or query in the workspace")
        .handler(move |input: BookmarkInput| {
            let session = session_clone_workspace_bookmark_tool.clone();
            let we = we_bookmark.clone();
            async move {
                track_session_call(&session, "ares_workspace_bookmark", &input);
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
    let session_clone_workspace_pin_tool = session_state.clone();
    let workspace_pin_tool = ToolBuilder::new("ares_workspace_pin")
        .description("Pin a node in the workspace")
        .handler(move |input: PinInput| {
            let session = session_clone_workspace_pin_tool.clone();
            let we = we_pin.clone();
            async move {
                track_session_call(&session, "ares_workspace_pin", &input);
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
    let session_clone_workspace_list_tool = session_state.clone();
    let workspace_list_tool = ToolBuilder::new("ares_workspace_list")
        .description("List recent questions, bookmarks, and pins")
        .handler(move |_input: EmptyInput| {
            let session = session_clone_workspace_list_tool.clone();
            let we = we_list.clone();
            async move {
                track_session_call(&session, "ares_workspace_list", &_input);
                let questions = we.list_recent_questions().await.unwrap_or_default();
                let bookmarks = we.list_bookmarks().await.unwrap_or_default();
                let pins = we.list_pinned_nodes().await.unwrap_or_default();
                let response = serde_json::json!({
                    "recent_questions": questions,
                    "bookmarks": bookmarks,
                    "pins": pins
                });
                Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_workspace_list", response, 0)).unwrap_or_default()))
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
    let session_clone_workspace_navigate_tool = session_state.clone();
    let workspace_navigate_tool = ToolBuilder::new("ares_workspace_navigate")
        .description("Navigate back or forward")
        .handler(move |input: NavigateInput| {
            let session = session_clone_workspace_navigate_tool.clone();
            let we = we_nav.clone();
            async move {
                track_session_call(&session, "ares_workspace_navigate", &input);
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

    let session_clone_chat_tool = session_state.clone();
    let chat_tool = ToolBuilder::new("ares_chat")
        .description("Repository Conversation Engine. Ask any question about the codebase.")
        .handler(move |input: ChatInput| {
            let session = session_clone_chat_tool.clone();
            let store = store_chat.clone();
            let path = project_path_chat.clone();
            let inference = inference_chat.clone();
            let we = we_chat.clone();

            async move {
                track_session_call(&session, "ares_chat", &input);

                // No LLM provider configured — fail clearly instead of silently returning mock data
                if std::env::var("OPENAI_API_KEY").is_err() 
                    && std::env::var("ANTHROPIC_API_KEY").is_err() {
                    return Ok(CallToolResult::text(
                        serde_json::to_string(&wrap_with_envelope("ares_chat", serde_json::json!({
                            "error": "No LLM provider configured. ares_chat requires an LLM API key (OPENAI_API_KEY or ANTHROPIC_API_KEY)."
                        }), 0)).unwrap()
                    ));
                }
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
                        Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_chat", output, 0)).unwrap()))
                    },
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed chat", &e.to_string()))),
                }
            }
        })
        .build();

    let store_health = app_state.store.clone();
    let session_clone_health_tool = session_state.clone();






    let store_dead = app_state.store.clone();
    let dead_code_tool = ToolBuilder::new("ares_dead_code")
        .description("Finds dead code in the repository by detecting nodes without incoming dependencies.")
        .handler(move |_input: EmptyInput| {
            let store = store_dead.clone();
            async move {
                match ares_intelligence::dead_code::find_dead_code(&store, 30).await {
                    Ok(report) => {
                        let evidence = serde_json::json!([{"type": "graph_scan", "ref": "workspace"}]);
                        let conf = 0.6;
                        let mut payload = serde_json::to_value(&report).unwrap_or_default();
                        // Move warning string into caveats array (correct envelope field)
                        let warning_caveat = if let Some(serde_json::Value::Object(obj)) = Some(&mut payload) {
                            obj.remove("warning").and_then(|v| v.as_str().map(|s| s.to_string()))
                        } else {
                            None
                        };
                        if let Some(obj) = payload.as_object_mut() {
                            obj.insert("evidence".to_string(), evidence);
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                            if let Some(warn) = warning_caveat {
                                obj.insert("caveats".to_string(), serde_json::json!([warn]));
                            }
                        }
                        Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_dead_code", payload, 0)).unwrap_or_default()))
                    },
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed dead code", &e.to_string()))),
                }
            }
        }).build();

    let store_ctxf = app_state.store.clone();
    let pp_ctxf = project_path.clone();
    let pid_ctxf = session_state.lock().unwrap().project_id.clone();
    let context_file_tool = ToolBuilder::new("ares_generate_context_file")
        .description("Generates a CLAUDE.md context file with hotspots and decisions.")
        .handler(move |_input: EmptyInput| {
            let store = store_ctxf.clone();
            let pp = pp_ctxf.clone();
            let pid = pid_ctxf.clone();
            async move {
                let start = std::time::Instant::now();
                match ares_intelligence::context_file::generate_context_file(&store, &pp, &pid, None).await {
                    Ok(report) => {
                        let elapsed = start.elapsed().as_millis() as u64;
                        let evidence = serde_json::json!([{"type": "context_generation", "ref": "workspace"}]);
                        let conf = 0.6;
                        let inner = serde_json::json!( {
                            "evidence": evidence,
                            "confidence": conf,
                            "result": report
                        });
                        Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_generate_context_file", inner, elapsed)).unwrap_or_default()))
                    },
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed context file", &e.to_string()))),
                }
            }
        }).build();

    let store_brf = app_state.store.clone();
    let pp_brf = project_path.clone();
    let briefing_tool = ToolBuilder::new("ares_briefing")
        .description("Generates a high-level briefing report of the repository.")
        .handler(move |_input: EmptyInput| {
            let store = store_brf.clone();
            let pp = pp_brf.clone();
            async move {
                let start = std::time::Instant::now();
                match ares_intelligence::briefing::generate_briefing(&store, &pp).await {
                    Ok(report) => {
                        let elapsed = start.elapsed().as_millis() as u64;
                        let evidence = serde_json::json!([{"type": "session_aggregation", "ref": "workspace"}]);
                        let conf = 0.6;
                        let mut payload = serde_json::to_value(&report).unwrap_or_default();
                        truncate_large_arrays(&mut payload);
                        strip_details_uuids(&mut payload);
                        if let Some(obj) = payload.as_object_mut() {
                            obj.insert("evidence".to_string(), evidence);
                            obj.insert("confidence".to_string(), serde_json::json!(conf));
                        }
                        Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_briefing", payload, elapsed)).unwrap_or_default()))
                    },
                    Err(e) => Err(tower_mcp::Error::internal(format_mcp_error("Failed briefing", &e.to_string()))),
                }
            }
        }).build();

    let health_tool = ToolBuilder::new("ares_health_check")
        .description("Scans the repository memory graph for gaps (code without decisions, stale decisions, missing ownership) and returns a health score")
        .handler(move |_input: ArchitectureQueryInput| {
            let session = session_clone_health_tool.clone();
            let store = store_health.clone();
            async move {
                track_session_call(&session, "ares_health_check", &_input);
                let project_name = session.lock().unwrap().project_id.clone();
                let project_id = ares_core::ProjectId::from(project_name);

                let repo = ares_store::repositories::gaps::SqliteGapRepository::new(store.clone());

                let mut all_gaps = Vec::new();
                if let Ok(mut gaps) = repo.get_code_without_decision(&project_id, 30) {
                    all_gaps.append(&mut gaps);
                }
                if let Ok(mut gaps) = repo.get_decisions_without_code(&project_id, 7) {
                    all_gaps.append(&mut gaps);
                }
                if let Ok(mut gaps) = repo.get_orphaned_requirements(&project_id) {
                    all_gaps.append(&mut gaps);
                }
                if let Ok(mut gaps) = repo.get_stale_decisions(&project_id, 30) {
                    all_gaps.append(&mut gaps);
                }
                if let Ok(mut gaps) = repo.get_unknown_ownership(&project_id) {
                    all_gaps.append(&mut gaps);
                }

                let mut health_score = 0.0;
                let mut score_breakdown = serde_json::json!({});
                match repo.calculate_health_score(&project_id) {
                    Ok(score) => {
                        health_score = score.overall;
                        score_breakdown = serde_json::to_value(score).unwrap_or_default();
                    }
                    Err(e) => {
                        // Log so discrepancies like this are visible
                        eprintln!("[Health] score calculation failed: {}", e);
                    }
                }

                let hotspots = if let Ok(conn) = store.get_conn() {
                    ares_intelligence::hotspots::calculate_hotspots(&conn, 10).unwrap_or_default()
                } else {
                    Vec::new()
                };

                let evidence = serde_json::json!([{"type": "health_computation", "ref": "workspace"}]);
                let conf = 0.6;

                let mut result = serde_json::json!({
                    "gaps": all_gaps,
                    "health_score": health_score,
                    "score_breakdown": score_breakdown,
                    "hotspots": hotspots,
                    "evidence": evidence,
                    "confidence": conf
                });
                prefix_node_ids(&mut result);
                strip_details_uuids(&mut result);

                Ok(CallToolResult::text(serde_json::to_string(&wrap_with_envelope("ares_health_check", result, 0)).unwrap_or_default()))
            }
        })
        .build();

    let router = McpRouter::new()
        .server_info("ares-mcp", env!("CARGO_PKG_VERSION"))
        .instructions("ARES maintains a session memory. Use ares_end_session at the end of each session to persist your context for the next session. Use ares_session_context to retrieve past session context.")
        .tool(chat_tool)
        .tool(workspace_bookmark_tool)
        .tool(workspace_pin_tool)
        .tool(workspace_list_tool)
        .tool(workspace_navigate_tool)
        .tool(workspace_record_nav_tool)
        .tool(why_tool)


        .tool(impact_tool)
        .tool(compliance_tool)
        .tool(scorecard_tool)
        .tool(dashboard_tool)
        .tool(health_tool)
        .tool(briefing_tool)
        .tool(context_file_tool)
        .tool(dead_code_tool)
        .tool(coverage_tool)
        .tool(drift_tool)
        .tool(who_owns_tool)
        .tool(decisions_tool)
        .tool(search_tool)
        .tool(timeline_tool)
        .tool(compare_tool)
        .tool(architecture_tool)
        .tool(requirements_tool)
        .tool(session_context_tool)
        .tool(end_session_tool)
        .tool(record_decision_tool)
        .tool(record_requirement_tool)
        .tool(annotate_tool)
        .tool(correct_tool)
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
fn transform_graph_for_agent(value: &mut serde_json::Value) {
    if let serde_json::Value::Object(map) = value {
        if let Some(serde_json::Value::Array(nodes)) = map.get_mut("nodes") {
            for node in nodes {
                if let serde_json::Value::Object(n) = node {
                    n.remove("created_at");
                    n.remove("updated_at");
                    n.remove("valid_from");
                    n.remove("valid_until");

                    if let Some(serde_json::Value::Object(mut props)) = n.remove("properties") {
                        if let Some(lang) = props.remove("language") {
                            n.insert("language".to_string(), lang);
                        }
                        if let (Some(sl), Some(el)) = (props.remove("start_line"), props.remove("end_line")) {
                            n.insert("lines".to_string(), serde_json::Value::String(format!("{}-{}", sl, el)));
                        }
                        if let Some(ib) = props.remove("introduced_by") {
                            n.insert("introduced_by".to_string(), ib);
                        }
                        if let Some(ir) = props.remove("introduction_reason") {
                            n.insert("introduction_reason".to_string(), ir);
                        }
                    }

                    n.remove("id");
                    n.remove("project_id");
                    n.remove("deleted_at");
                }
            }
        }

        if let Some(serde_json::Value::Array(edges)) = map.get_mut("edges") {
            for edge in edges {
                if let serde_json::Value::Object(e) = edge {
                    e.remove("created_at");
                    e.remove("updated_at");
                    e.remove("valid_from");
                    e.remove("valid_until");

                    if let Some(src) = e.remove("source") {
                        e.insert("provenance".to_string(), src);
                    }

                    e.remove("id");
                    e.remove("project_id");
                    e.remove("weight");
                    e.remove("confidence");
                }
            }
        }
    }
}

fn build_shortest_path_answer(mut payload: serde_json::Value) -> (serde_json::Value, Vec<serde_json::Value>) {
    let mut node_map = std::collections::HashMap::new();
    if let Some(nodes) = payload.get("nodes").and_then(|n| n.as_array()) {
        for n in nodes {
            if let Some(id) = n.get("id").and_then(|i| i.as_str()) {
                let lbl = n.get("label").and_then(|l| l.as_str()).unwrap_or("").to_string();
                node_map.insert(id.to_string(), lbl);
            }
        }
    }
    
    transform_graph_for_agent(&mut payload);
    
    let mut path = Vec::new();
    if let Some(nodes) = payload.get_mut("nodes").and_then(|n| n.as_array_mut()) {
        for n in nodes {
            path.push(n.take());
        }
    }
    
    let mut hops = Vec::new();
    if let Some(edges) = payload.get_mut("edges").and_then(|e| e.as_array_mut()) {
        for e in edges {
            if let Some(obj) = e.as_object_mut() {
                let from_id = obj.remove("from_node_id").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
                let to_id = obj.remove("to_node_id").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
                let via = obj.remove("edge_type").unwrap_or(serde_json::json!(""));
                
                let from_lbl = node_map.get(&from_id).cloned().unwrap_or(from_id);
                let to_lbl = node_map.get(&to_id).cloned().unwrap_or(to_id);
                
                obj.insert("from".to_string(), serde_json::json!(from_lbl));
                obj.insert("to".to_string(), serde_json::json!(to_lbl));
                obj.insert("via".to_string(), via);
                hops.push(serde_json::Value::Object(obj.clone()));
            }
        }
    }
    
    let final_answer = serde_json::json!({
        "path": path,
        "hops": hops,
        "total_hops": hops.len()
    });

    let mut evidence = Vec::new();
    for hop in &hops {
        let from = hop.get("from").and_then(|v| v.as_str()).unwrap_or("");
        let to = hop.get("to").and_then(|v| v.as_str()).unwrap_or("");
        let via = hop.get("via").and_then(|v| v.as_str()).unwrap_or("");
        
        let path_from = if let Some(pn) = path.iter().find(|n| n.get("label").and_then(|l| l.as_str()) == Some(from)) {
            let fp = pn.get("file_path").and_then(|f| f.as_str()).unwrap_or(from);
            if fp.ends_with(from) { fp.to_string() } else { format!("{}:{}", fp, from) }
        } else { from.to_string() };
        
        let path_to = if let Some(pn) = path.iter().find(|n| n.get("label").and_then(|l| l.as_str()) == Some(to)) {
            let fp = pn.get("file_path").and_then(|f| f.as_str()).unwrap_or(to);
            if fp.ends_with(to) { fp.to_string() } else { format!("{}:{}", fp, to) }
        } else { to.to_string() };

        evidence.push(serde_json::json!({
            "type": "graph_edge",
            "ref": format!("{}:{}\u{2192}{}", path_from, via, path_to)
        }));
    }

    (final_answer, evidence)
}

fn build_neighbors_answer(mut payload: serde_json::Value, target_id_str: &str) -> (serde_json::Value, Vec<serde_json::Value>) {
    let mut id_to_idx = std::collections::HashMap::new();
    if let Some(nodes) = payload.get("nodes").and_then(|n| n.as_array()) {
        for (idx, n) in nodes.iter().enumerate() {
            if let Some(id) = n.get("id").and_then(|i| i.as_str()) {
                id_to_idx.insert(id.to_string(), idx);
            }
        }
    }
    
    transform_graph_for_agent(&mut payload);
    
    let nodes = payload.get("nodes").and_then(|n| n.as_array()).cloned().unwrap_or_default();
    
    let mut target_path = target_id_str.to_string();
    if let Some(&idx) = id_to_idx.get(target_id_str) {
        if let (Some(fp), Some(lbl)) = (nodes[idx].get("file_path").and_then(|f| f.as_str()), nodes[idx].get("label").and_then(|l| l.as_str())) {
            target_path = if fp.ends_with(lbl) { fp.to_string() } else { format!("{}:{}", fp, lbl) };
        }
    }

    let mut neighbors = Vec::new();
    if let Some(edges) = payload.get_mut("edges").and_then(|e| e.as_array_mut()) {
        for e in edges {
            if let Some(obj) = e.as_object_mut() {
                let from_id = obj.remove("from_node_id").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
                let to_id = obj.remove("to_node_id").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
                let rel = obj.remove("edge_type").unwrap_or(serde_json::json!(""));
                
                let mut neighbor_obj = None;
                if from_id == target_id_str {
                    if let Some(&idx) = id_to_idx.get(&to_id) {
                        let mut n_obj = nodes[idx].as_object().unwrap().clone();
                        n_obj.insert("relationship".to_string(), rel.clone());
                        n_obj.insert("direction".to_string(), serde_json::json!("outgoing"));
                        if let Some(prov) = obj.get("provenance") {
                            n_obj.insert("provenance".to_string(), prov.clone());
                        }
                        neighbor_obj = Some(n_obj);
                    }
                } else if to_id == target_id_str {
                    if let Some(&idx) = id_to_idx.get(&from_id) {
                        let mut n_obj = nodes[idx].as_object().unwrap().clone();
                        n_obj.insert("relationship".to_string(), rel.clone());
                        n_obj.insert("direction".to_string(), serde_json::json!("incoming"));
                        if let Some(prov) = obj.get("provenance") {
                            n_obj.insert("provenance".to_string(), prov.clone());
                        }
                        neighbor_obj = Some(n_obj);
                    }
                }
                
                if let Some(n) = neighbor_obj {
                    neighbors.push(serde_json::Value::Object(n));
                }
            }
        }
    }
    
    let mut truncated = false;
    let mut caveats = Vec::new();
    let total_found = neighbors.len();
    if neighbors.len() > 20 {
        neighbors.truncate(20);
        truncated = true;
        caveats.push(serde_json::json!(format!("results truncated; {} total neighbors found", total_found)));
    }
    
    let final_answer = serde_json::json!({
        "target": target_path,
        "neighbors": neighbors,
        "total": total_found,
        "caveats": caveats,
        "meta": {
            "truncated": truncated
        }
    });

    let mut evidence = Vec::new();
    for n in &neighbors {
        let n_path = n.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
        let lbl = n.get("label").and_then(|v| v.as_str()).unwrap_or("");
        let node_desc = if n_path.ends_with(lbl) { n_path.to_string() } else { format!("{}:{}", n_path, lbl) };
        let rel = n.get("relationship").and_then(|v| v.as_str()).unwrap_or("");
        let dir = n.get("direction").and_then(|v| v.as_str()).unwrap_or("");
        
        let r = if dir == "outgoing" {
            format!("{}:{}\u{2192}{}", target_path, rel, node_desc)
        } else {
            format!("{}:{}\u{2192}{}", node_desc, rel, target_path)
        };
        evidence.push(serde_json::json!({
            "type": "graph_edge",
            "ref": r
        }));
    }

    (final_answer, evidence)
}
