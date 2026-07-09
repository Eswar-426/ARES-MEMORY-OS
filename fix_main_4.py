import re

file_path = 'crates/ares-mcp/src/main.rs'
with open(file_path, 'r', encoding='utf-8') as f:
    code = f.read()

# Fix who_tool
bad_who = '''        .handler(move |input: MemoryQueryInput| {
            let facade = facade_who.clone();
            async move {
                let store = store_who.clone();'''
good_who = '''        .handler(move |input: MemoryQueryInput| {
            let facade = facade_who.clone();
            let store = store_who.clone();
            async move {'''
code = code.replace(bad_who, good_who)

# Fix impact_tool
bad_impact = '''        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_impact_tool.clone();
            let facade = intelligence_facade_impact.clone();
            let project_id = project_id_str_impact.clone();
            async move {
                let store = store_impact.clone();'''
good_impact = '''        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_impact_tool.clone();
            let facade = intelligence_facade_impact.clone();
            let project_id = project_id_str_impact.clone();
            let store = store_impact.clone();
            async move {'''
code = code.replace(bad_impact, good_impact)

# Fix drift_tool
bad_drift = '''        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_drift_tool.clone();
            let facade = intelligence_facade_drift.clone();
            let project_id = project_id_str_drift.clone();
            async move {
                let store = store_drift_new.clone();'''
good_drift = '''        .handler(move |input: MemoryQueryInput| {
            let session = session_clone_drift_tool.clone();
            let facade = intelligence_facade_drift.clone();
            let project_id = project_id_str_drift.clone();
            let store = store_drift_new.clone();
            async move {'''
code = code.replace(bad_drift, good_drift)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(code)

print('Fixed async move closure captures.')
