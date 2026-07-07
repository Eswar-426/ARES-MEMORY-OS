import re

with open('crates/ares-cli/src/commands/ingest.rs', 'r', encoding='utf-8') as f:
    text = f.read()

patch_code = """
            let now_micros = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64;
            for pr_dec in git_memory.pr_decisions {
                let requires_human_review = pr_dec.confidence < 0.79;
                let dec_id = ares_core::NodeId::from(format!("decision:pr:{}", pr_dec.commit_hash));
                let props = serde_json::json!({
                    "source": "inferred_pr",
                    "pr_number": pr_dec.pr_number,
                    "confidence": pr_dec.confidence,
                    "requires_human_review": requires_human_review,
                    "extracted_heading": pr_dec.extracted_heading,
                    "created_at": now_micros,
                    "decision": pr_dec.description,
                    "title": pr_dec.title,
                });
                
                let dec_node = ares_core::GraphNode {
                    id: dec_id.clone(),
                    project_id: project_id.clone(),
                    node_type: ares_core::NodeType::Decision,
                    label: pr_dec.title.clone(),
                    properties: props,
                    file_path: None,
                    created_at: now_micros,
                    updated_at: now_micros,
                    deleted_at: None,
                };
                
                if let Err(e) = repo.upsert_node(dec_node) {
                    println!("DEBUG: Failed to upsert PR decision node: {:?}", e);
                }
                
                for file_path in pr_dec.touched_files {
                    let canonical = ares_core::canonicalize_node_id(&file_path);
                    let target_id = if let Some(scanner_id) = path_to_scanner_id.get(&canonical) {
                        ares_core::NodeId::from(scanner_id.as_str())
                    } else {
                        continue;
                    };
                    
                    let edge = ares_core::GraphEdge {
                        id: format!("{}-relatedto-{}", dec_id.as_str(), target_id.as_str()),
                        project_id: project_id.clone(),
                        from_node_id: dec_id.clone(),
                        to_node_id: target_id,
                        edge_type: ares_core::EdgeType::RelatedTo,
                        weight: 1.0,
                        confidence: pr_dec.confidence,
                        source: "agent_decision".to_string(),
                        valid_from: now_micros,
                        valid_until: None,
                        created_at: now_micros,
                    };
                    
                    if let Err(e) = repo.upsert_edge(edge) {
                        println!("DEBUG: Failed to upsert PR decision edge: {:?}", e);
                    }
                }
            }
        };
"""

idx = text.find('        };\n\n        match git_extractor.extract_metadata_only(&project_id) {')
text = text[:idx] + patch_code + '\n' + text[idx+11:]

with open('crates/ares-cli/src/commands/ingest.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print('Updated ingest.rs')
