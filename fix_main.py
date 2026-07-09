import re

file_path = 'crates/ares-mcp/src/main.rs'
with open(file_path, 'r', encoding='utf-8') as f:
    code = f.read()

# 1. Update MemoryQueryInput
memory_query_old = '''struct MemoryQueryInput {
    id: String,
}'''
memory_query_new = '''struct MemoryQueryInput {
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
}'''
code = code.replace(memory_query_old, memory_query_new)

# 2. Update ProjectQueryInput
project_query_old = '''struct ProjectQueryInput {
    project_id: String,
}'''
project_query_new = '''struct ProjectQueryInput {
    project_id: Option<String>,
}'''
code = code.replace(project_query_old, project_query_new)

# 3. Update TraceabilityInput
traceability_query_old = '''struct TraceabilityInput {
    entity_id: String,
    depth: Option<usize>,
}'''
traceability_query_new = '''struct TraceabilityInput {
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
}'''
code = code.replace(traceability_query_old, traceability_query_new)

# 4. Update ares_coverage
coverage_old = '''                let project_id = ares_core::ProjectId::from(input.project_id);'''
coverage_new = '''                let project_name = input.project_id.clone().unwrap_or_else(|| session.lock().unwrap().project_id.clone());
                let project_id = ares_core::ProjectId::from(project_name);'''
code = code.replace(coverage_old, coverage_new)

# 5. Fix tool handlers for MemoryQueryInput and TraceabilityInput

# ares_why_exists
why_old = '''    let why_tool = ToolBuilder::new("ares_why_exists")
        .description("Explains why a specific entity exists in the ARES memory graph")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_why_tool.clone();
            let facade = intelligence_facade_why.clone();
            let project_id = project_id_str.clone();

            async move {
                track_session_call(&session, "ares_why_exists", &input);
                let id = ares_core::canonicalize_node_id(&input.id);'''
why_new = '''    let store_why = app_state.store.clone();
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
                let id = ares_core::canonicalize_node_id(&id_str);'''
code = code.replace(why_old, why_new)

# ares_evolution
evolution_old = '''    let evolution_tool = ToolBuilder::new("ares_evolution")
        .description("Retrieves the evolutionary timeline of an entity")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_evolution_tool.clone();
            let facade = facade_evolution.clone();
            async move {
                track_session_call(&session, "ares_evolution", &input);
                let id = ares_core::canonicalize_node_id(&input.id);'''
evolution_new = '''    let store_evolution = app_state.store.clone();
    let evolution_tool = ToolBuilder::new("ares_evolution")
        .description("Retrieves the evolutionary timeline of an entity")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_evolution_tool.clone();
            let facade = facade_evolution.clone();
            let store = store_evolution.clone();
            async move {
                track_session_call(&session, "ares_evolution", &input);
                let id_str = match input.resolve_id(&store) {
                    Ok(i) => i,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                };
                let id = ares_core::canonicalize_node_id(&id_str);'''
code = code.replace(evolution_old, evolution_new)

# ares_impact
impact_old = '''    let impact_tool = ToolBuilder::new("ares_impact")
        .description("Analyzes the blast radius of changing a specific entity")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_impact_tool.clone();
            let facade = intelligence_facade_impact.clone();
            let project_id = project_id_str_impact.clone();
            async move {
                track_session_call(&session, "ares_impact", &input);
                let id = ares_core::canonicalize_node_id(&input.id);'''
impact_new = '''    let store_impact = app_state.store.clone();
    let impact_tool = ToolBuilder::new("ares_impact")
        .description("Analyzes the blast radius of changing a specific entity")
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
                let id = ares_core::canonicalize_node_id(&id_str);'''
code = code.replace(impact_old, impact_new)

# ares_drift
drift_old = '''    let drift_tool = ToolBuilder::new("ares_drift")
        .description("Analyzes architectural drift of an entity over time vs requirements")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_drift_tool.clone();
            let facade = intelligence_facade_drift.clone();
            let project_id = project_id_str_drift.clone();
            async move {
                track_session_call(&session, "ares_drift", &input);
                let id = ares_core::canonicalize_node_id(&input.id);'''
drift_new = '''    let store_drift = app_state.store.clone();
    let drift_tool = ToolBuilder::new("ares_drift")
        .description("Analyzes architectural drift of an entity over time vs requirements")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_drift_tool.clone();
            let facade = intelligence_facade_drift.clone();
            let project_id = project_id_str_drift.clone();
            let store = store_drift.clone();
            async move {
                track_session_call(&session, "ares_drift", &input);
                let id_str = match input.resolve_id(&store) {
                    Ok(i) => i,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                };
                let id = ares_core::canonicalize_node_id(&id_str);'''
code = code.replace(drift_old, drift_new)

# ares_traceability
trace_old = '''    let traceability_tool = ToolBuilder::new("ares_traceability")
        .description("Evaluates traceability relationships upstream and downstream")
        .handler(move |input: TraceabilityInput| {
            let session = session_clone_traceability_tool.clone();
            let facade = intelligence_facade_trace.clone();
            let project_id = project_id_str_trace.clone();
            async move {
                track_session_call(&session, "ares_traceability", &input);
                let id = ares_core::canonicalize_node_id(&input.entity_id);'''
trace_new = '''    let store_traceability = app_state.store.clone();
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
                let id = ares_core::canonicalize_node_id(&id_str);'''
code = code.replace(trace_old, trace_new)

# ares_graph_metadata
graph_meta_old = '''    let graph_metadata_tool = ToolBuilder::new("ares_graph_metadata")
        .description("Retrieves graph metadata and properties for an entity")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_graph_metadata_tool.clone();
            let store = store_graph_metadata.clone();
            async move {
                track_session_call(&session, "ares_graph_metadata", &input);
                let node_id_str = ares_core::canonicalize_node_id(&input.id);'''
graph_meta_new = '''    let graph_metadata_tool = ToolBuilder::new("ares_graph_metadata")
        .description("Retrieves graph metadata and properties for an entity")
        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_graph_metadata_tool.clone();
            let store = store_graph_metadata.clone();
            async move {
                track_session_call(&session, "ares_graph_metadata", &input);
                let id_str = match input.resolve_id(&store) {
                    Ok(i) => i,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                };
                let node_id_str = ares_core::canonicalize_node_id(&id_str);'''
code = code.replace(graph_meta_old, graph_meta_new)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(code)

print('Updated crates/ares-mcp/src/main.rs successfully.')
