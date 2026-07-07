import re

with open('crates/ares-mcp/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('let router = McpRouter::new()')

health_tool_code = """
    let store_health = app_state.store.clone();
    let session_clone_health_tool = session_state.clone();
    let health_tool = ToolBuilder::new("ares_health_check")
        .description("Scans the repository memory graph for gaps (code without decisions, stale decisions, missing ownership) and returns a health score")
        .handler(move |_input: ProjectQueryInput| {
            let session = session_clone_health_tool.clone();
            let store = store_health.clone();
            async move {
                track_session_call(&session, "ares_health_check", &_input);
                let project_name = session.lock().unwrap().project_id.clone();
                let project_id = ares_core::ProjectId::from(project_name);
                
                let repo = ares_store::repositories::gaps::SqliteGapRepository::new(store);
                
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
                
                let penalty = all_gaps.len() as f64 * 5.0;
                let health_score = (100.0 - penalty).max(0.0);
                
                let result = serde_json::json!({
                    "gaps": all_gaps,
                    "health_score": health_score
                });
                
                Ok(CallToolResult::text(result.to_string()))
            }
        })
        .build();
"""

text = text[:idx] + health_tool_code + '\n' + text[idx:]
text = text.replace('.tool(dashboard_tool)', '.tool(dashboard_tool)\n        .tool(health_tool)')

with open('crates/ares-mcp/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print('Patched main.rs')
