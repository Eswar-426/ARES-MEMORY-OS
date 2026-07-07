with open('crates/ares-mcp/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

structs = """
// === Phase 3: Task 3.2 — Agent Memory Write API ===

#[derive(Debug, Deserialize, JsonSchema)]
struct RecordDecisionInput {
    title: String,
    description: String,
    status: String,
    impacted_paths: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RecordRequirementInput {
    title: String,
    description: String,
    priority: String,
    satisfies_paths: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AnnotateInput {
    target_path: String,
    key: String,
    value: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CorrectInput {
    target_path: String,
    correction_notes: String,
}
"""

if 'RecordDecisionInput' not in text:
    import re
    # insert structs after RequiremQueryInput
    idx = text.find('#[derive(Debug, Deserialize, JsonSchema)]\nstruct RequiremQueryInput')
    if idx == -1:
        idx = text.find('struct RequirementsQueryInput')
    if idx == -1:
        print("Couldn't find struct to insert after")
        idx = text.find('fn main')
    text = text[:idx] + structs + "\n" + text[idx:]


handlers = """
    // --- Task 3.2: Agent Memory Write API ---
    let store_rec_dec = store.clone();
    let pp_rec_dec = pp.clone();
    let record_decision_tool = ToolBuilder::new("ares_record_decision")
        .description("Record an architectural decision and link it to impacted files")
        .handler(move |input: RecordDecisionInput| {
            let store_arc = store_rec_dec.clone();
            let pp_local = pp_rec_dec.clone();
            async move {
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp_local).file_name().unwrap_or_default().to_string_lossy().to_string();
                let project_id = ares_core::ProjectId::from(project_name);
                
                let node_id = ares_core::NodeId::new();
                let now = ares_core::now_micros();
                
                let properties = serde_json::json!({
                    "source": "agent",
                    "description": input.description,
                    "status": input.status,
                    "confidence": 1.0
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
                    return Ok(CallToolResult::text(format!("Failed to record decision: {}", e)));
                }
                
                let mut linked_files = Vec::new();
                for path in input.impacted_paths {
                    if let Ok(file_id_str) = repo.get_id_by_path(&path) {
                        let file_id = ares_core::NodeId::from(file_id_str);
                        let edge = ares_core::GraphEdge {
                            id: ares_core::new_id(),
                            project_id: project_id.clone(),
                            from_node_id: node_id.clone(),
                            to_node_id: file_id,
                            edge_type: ares_core::EdgeType::RelatedTo,
                            weight: 1.0,
                            confidence: 1.0,
                            source: "agent_decision".to_string(),
                            valid_from: now,
                            valid_until: None,
                            created_at: now,
                        };
                        if repo.upsert_edge(edge).is_ok() {
                            linked_files.push(path);
                        }
                    }
                }
                
                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "result": "Decision recorded",
                    "decision_id": node_id.as_str(),
                    "linked_files": linked_files
                })).unwrap_or_default()))
            }
        });

    let store_rec_req = store.clone();
    let pp_rec_req = pp.clone();
    let record_requirement_tool = ToolBuilder::new("ares_record_requirement")
        .description("Record a business or technical requirement and link it to files")
        .handler(move |input: RecordRequirementInput| {
            let store_arc = store_rec_req.clone();
            let pp_local = pp_rec_req.clone();
            async move {
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                let project_name = std::path::Path::new(&pp_local).file_name().unwrap_or_default().to_string_lossy().to_string();
                let project_id = ares_core::ProjectId::from(project_name);
                
                let node_id = ares_core::NodeId::new();
                let now = ares_core::now_micros();
                
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
                    return Ok(CallToolResult::text(format!("Failed to record requirement: {}", e)));
                }
                
                let mut linked_files = Vec::new();
                for path in input.satisfies_paths {
                    if let Ok(file_id_str) = repo.get_id_by_path(&path) {
                        let file_id = ares_core::NodeId::from(file_id_str);
                        let edge = ares_core::GraphEdge {
                            id: ares_core::new_id(),
                            project_id: project_id.clone(),
                            from_node_id: file_id,
                            to_node_id: node_id.clone(),
                            edge_type: ares_core::EdgeType::RelatedTo,
                            weight: 1.0,
                            confidence: 1.0,
                            source: "agent_requirement".to_string(),
                            valid_from: now,
                            valid_until: None,
                            created_at: now,
                        };
                        if repo.upsert_edge(edge).is_ok() {
                            linked_files.push(path);
                        }
                    }
                }
                
                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "result": "Requirement recorded",
                    "requirement_id": node_id.as_str(),
                    "linked_files": linked_files
                })).unwrap_or_default()))
            }
        });

    let store_ann = store.clone();
    let annotate_tool = ToolBuilder::new("ares_annotate")
        .description("Annotate a file or node by adding a key-value property")
        .handler(move |input: AnnotateInput| {
            let store_arc = store_ann.clone();
            async move {
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                
                if let Ok(file_id_str) = repo.get_id_by_path(&input.target_path) {
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
                            node.updated_at = ares_core::now_micros();
                            
                            if let Ok(_) = repo.upsert_node(node) {
                                return Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                                    "result": "Annotation added",
                                    "target": input.target_path,
                                    "key": input.key
                                })).unwrap_or_default()));
                            }
                        }
                    }
                }
                
                Ok(CallToolResult::text(serde_json::to_string(&serde_json::json!({
                    "error": "Failed to add annotation: node not found"
                })).unwrap_or_default()))
            }
        });

    let store_corr = store.clone();
    let correct_tool = ToolBuilder::new("ares_correct")
        .description("Correct a node's properties manually")
        .handler(move |input: CorrectInput| {
            let store_arc = store_corr.clone();
            async move {
                let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store_arc.clone());
                
                if let Ok(file_id_str) = repo.get_id_by_path(&input.target_path) {
                    let file_id = ares_core::NodeId::from(file_id_str);
                    if let Ok(Some(mut node)) = repo.get_node(&file_id) {
                        if let Some(obj) = node.properties.as_object_mut() {
                            let mut corrections = obj.remove("corrections").unwrap_or_else(|| serde_json::json!([]));
                            if let Some(arr) = corrections.as_array_mut() {
                                arr.push(serde_json::json!({
                                    "timestamp": ares_core::now_micros(),
                                    "note": input.correction_notes
                                }));
                            }
                            obj.insert("corrections".to_string(), corrections);
                            node.updated_at = ares_core::now_micros();
                            
                            if let Ok(_) = repo.upsert_node(node) {
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
        });
"""

if 'record_decision_tool' not in text:
    idx = text.find('let gaps_tool = ToolBuilder')
    if idx == -1:
        idx = text.find('let router = McpRouter')
    text = text[:idx] + handlers + "\n\n    " + text[idx:]

router_calls = """        .tool(record_decision_tool)
        .tool(record_requirement_tool)
        .tool(annotate_tool)
        .tool(correct_tool)"""

if 'record_decision_tool' not in text.split('let router = McpRouter')[1]:
    idx = text.find('.tool(requirements_tool)')
    if idx != -1:
        end_idx = text.find('\n', idx)
        text = text[:end_idx] + '\n' + router_calls + text[end_idx:]

with open('crates/ares-mcp/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
