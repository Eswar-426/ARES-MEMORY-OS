import re

file_path = 'crates/ares-mcp/src/main.rs'
with open(file_path, 'r', encoding='utf-8') as f:
    code = f.read()

# 1. Fix all `ares_core::canonicalize_node_id(&input.id)` to use resolve_id
pattern_canonicalize = r'let ([\w_]+) = ares_core::canonicalize_node_id\(&input\.id\);'

def repl_canonicalize(match):
    var_name = match.group(1)
    return f'''let {var_name}_str = match input.resolve_id(&store) {{
                    Ok(i) => i,
                    Err(e) => return Err(tower_mcp::Error::invalid_params(e)),
                }};
                let {var_name} = ares_core::canonicalize_node_id(&{var_name}_str);'''

code = re.sub(pattern_canonicalize, repl_canonicalize, code)

# 2. Fix `store` variables in the handlers that just got resolve_id

# who_owns
who_old = r'(\.handler\(move \|input: MemoryQueryInput\| \{\s+let facade = facade_who\.clone\(\);\s+async move \{)'
who_new = r'let store_who = app_state.store.clone();\n        \1\n                let store = store_who.clone();'
code = re.sub(who_old, who_new, code)

# impact
impact_old = r'(\.handler\(move \|input: MemoryQueryInput\| \{\s+let session = session_clone_impact_tool\.clone\(\);\s+let facade = intelligence_facade_impact\.clone\(\);\s+let project_id = project_id_str_impact\.clone\(\);\s+async move \{)'
impact_new = r'let store_impact = app_state.store.clone();\n        \1\n                let store = store_impact.clone();'
code = re.sub(impact_old, impact_new, code)

# drift
drift_old = r'(\.handler\(move \|input: MemoryQueryInput\| \{\s+let session = session_clone_drift_tool\.clone\(\);\s+let facade = intelligence_facade_drift\.clone\(\);\s+let project_id = project_id_str_drift\.clone\(\);\s+async move \{)'
drift_new = r'let store_drift_new = app_state.store.clone();\n        \1\n                let store = store_drift_new.clone();'
code = re.sub(drift_old, drift_new, code)

# graph_metadata
graph_meta_old = r'(\.handler\(move \|input: MemoryQueryInput\| \{\s+let session = session_clone_graph_metadata_tool\.clone\(\);\s+let store = store_graph_metadata\.clone\(\);\s+async move \{)'
# store is already cloned for graph_metadata! We don't need to inject it.
# So we do nothing for graph_metadata's store.

# 3. Fix ProjectId::from(input.project_id)
# There's a case in `get_scorecard(&ares_core::ProjectId::from(input.project_id))`
dashboard_old = r'get_scorecard\(&ares_core::ProjectId::from\(input\.project_id\)\)'
dashboard_new = r'get_scorecard(&ares_core::ProjectId::from(input.project_id.clone().unwrap_or_else(|| session.lock().unwrap().project_id.clone())))'
code = re.sub(dashboard_old, dashboard_new, code)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(code)

print('Updated crates/ares-mcp/src/main.rs successfully.')
