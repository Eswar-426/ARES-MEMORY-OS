import re

file_path = 'crates/ares-mcp/src/main.rs'
with open(file_path, 'r', encoding='utf-8') as f:
    code = f.read()

# Fix who_tool
bad_who = '''    let who_tool = ToolBuilder::new("ares_who_owns")
        .description("Identifies ownership and authorship information for an entity")
        let store_who = app_state.store.clone();
        .handler(move |input: MemoryQueryInput| {'''
good_who = '''    let store_who = app_state.store.clone();
    let who_tool = ToolBuilder::new("ares_who_owns")
        .description("Identifies ownership and authorship information for an entity")
        .handler(move |input: MemoryQueryInput| {'''
code = code.replace(bad_who, good_who)

# Fix impact_tool
bad_impact = '''    let impact_tool = ToolBuilder::new("ares_impact")
        .description("Performs read-only dependency analysis to determine what downstream components break if this entity is modified. Use this for general blast-radius queries without mutating the graph.")
        let store_impact = app_state.store.clone();
        .handler(move |input: MemoryQueryInput| {'''
good_impact = '''    let store_impact = app_state.store.clone();
    let impact_tool = ToolBuilder::new("ares_impact")
        .description("Performs read-only dependency analysis to determine what downstream components break if this entity is modified. Use this for general blast-radius queries without mutating the graph.")
        .handler(move |input: MemoryQueryInput| {'''
code = code.replace(bad_impact, good_impact)

# Fix drift_tool
bad_drift = '''    let drift_tool = ToolBuilder::new("ares_drift")
        .description("Evaluates structural drift for a given file")
        let store_drift_new = app_state.store.clone();
        .handler(move |input: MemoryQueryInput| {'''
good_drift = '''    let store_drift_new = app_state.store.clone();
    let drift_tool = ToolBuilder::new("ares_drift")
        .description("Evaluates structural drift for a given file")
        .handler(move |input: MemoryQueryInput| {'''
code = code.replace(bad_drift, good_drift)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(code)

print('Fixed syntax errors.')
