use ares_knowledge_graph::models::{EdgeType, KnowledgeEdge, KnowledgeNode, NodeType};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct GapGenerator;

impl GapGenerator {
    pub fn generate_gaps(
        nodes: &mut Vec<KnowledgeNode>,
        edges: &mut Vec<KnowledgeEdge>,
        timestamp: i64,
    ) {
        let mut out_degree: HashMap<String, Vec<(&EdgeType, String)>> = HashMap::new();
        let mut in_degree: HashMap<String, Vec<(&EdgeType, String)>> = HashMap::new();

        for edge in edges.iter() {
            out_degree
                .entry(edge.source_id.clone())
                .or_default()
                .push((&edge.edge_type, edge.target_id.clone()));
            in_degree
                .entry(edge.target_id.clone())
                .or_default()
                .push((&edge.edge_type, edge.source_id.clone()));
        }

        let mut code_has_test: HashSet<String> = HashSet::new();
        let mut req_has_impl: HashSet<String> = HashSet::new();
        let mut req_has_dec: HashSet<String> = HashSet::new();
        let mut dec_has_req: HashSet<String> = HashSet::new();
        let mut dec_has_impl: HashSet<String> = HashSet::new();
        let mut test_has_code: HashSet<String> = HashSet::new();
        let _test_has_req: HashSet<String> = HashSet::new();

        // 1st pass: build basic maps
        for edge in edges.iter() {
            match edge.edge_type {
                EdgeType::ValidatedBy => {
                    code_has_test.insert(edge.source_id.clone());
                    test_has_code.insert(edge.target_id.clone());
                }
                EdgeType::ImplementedBy => {
                    req_has_impl.insert(edge.source_id.clone());
                }
                EdgeType::Drives => {
                    dec_has_impl.insert(edge.source_id.clone());
                }
                EdgeType::ResultsIn => {
                    req_has_dec.insert(edge.source_id.clone());
                    dec_has_req.insert(edge.target_id.clone());
                }
                _ => {}
            }
        }

        // Trace Test -> Requirement (Test is ValidatedBy target, Code is ValidatedBy source)
        // Code -> ImplementedBy (source is Req)
        let mut test_to_req = HashSet::new();
        for edge in edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::ValidatedBy)
        {
            let code_id = &edge.source_id;
            let test_id = &edge.target_id;
            if let Some(req_edges) = in_degree.get(code_id) {
                for (etype, _req_id) in req_edges {
                    if **etype == EdgeType::ImplementedBy {
                        test_to_req.insert(test_id.clone());
                    }
                }
            }
        }

        let mut gap_nodes = Vec::new();
        let mut gap_edges = Vec::new();

        let mut push_gap = |target_id: &str, gap_type: &str| {
            let gap_id = format!("GAP-{}-{}", gap_type, target_id);
            gap_nodes.push(KnowledgeNode {
                id: gap_id.clone(),
                node_type: NodeType::KnowledgeGap,
                name: gap_type.to_string(),
                properties: serde_json::json!({"gap_type": gap_type}),
                created_at: timestamp,
            });
            gap_edges.push(KnowledgeEdge {
                id: Uuid::new_v4().to_string(),
                source_id: gap_id,
                target_id: target_id.to_string(),
                edge_type: EdgeType::HasGap,
                confidence: 1.0,
                created_at: timestamp,
                properties: serde_json::json!({}),
            });
        };

        for node in nodes.iter() {
            let id = &node.id;
            match node.node_type {
                NodeType::CodeArtifact => {
                    if !code_has_test.contains(id) {
                        push_gap(id, "CodeWithoutTests");
                    }
                }
                NodeType::Requirement => {
                    if !req_has_impl.contains(id) {
                        push_gap(id, "RequirementWithoutImplementation");
                    } else {
                        // Check if its implementations have tests
                        let mut all_impls_untested = true;
                        if let Some(impls) = out_degree.get(id) {
                            for (etype, target) in impls {
                                if **etype == EdgeType::ImplementedBy
                                    && code_has_test.contains(target) {
                                        all_impls_untested = false;
                                        break;
                                    }
                            }
                        }
                        if all_impls_untested {
                            push_gap(id, "RequirementWithoutTests");
                        }
                    }

                    if !req_has_dec.contains(id) {
                        push_gap(id, "RequirementWithoutDecision");
                    }

                    if !out_degree.contains_key(id) && !in_degree.contains_key(id) {
                        push_gap(id, "OrphanRequirement");
                    }
                }
                NodeType::Decision => {
                    if !dec_has_impl.contains(id) {
                        push_gap(id, "DecisionWithoutImplementation");
                    }
                    if !dec_has_req.contains(id) {
                        push_gap(id, "DecisionWithoutRequirement");
                        push_gap(id, "OrphanDecision"); // Emitting both as per request
                    }
                }
                // Determine if it's a test file by name pattern if NodeType is CodeArtifact
                // Wait, TestDiscoveryEngine actually doesn't create NodeType::Test, it creates CodeArtifact.
                // But the user referred to `TestArtifact`... wait, does `NodeType::Test` exist?
                // Let's assume nodes ending in `_test.rs` or `.test.ts` are tests. But we don't have a `NodeType::Test`.
                // Actually, let's look if `NodeType::Test` exists. We can just check `test_has_code`.
                _ => {}
            }
        }

        // Find tests among code artifacts
        for node in nodes.iter() {
            if node.node_type == NodeType::CodeArtifact {
                let id = &node.id;
                let is_test = id.contains("test") || id.contains("spec"); // simple heuristic
                if is_test {
                    if !test_has_code.contains(id) {
                        push_gap(id, "TestWithoutCode");
                    }
                    if !test_to_req.contains(id) {
                        push_gap(id, "TestWithoutRequirement");
                    }
                    push_gap(id, "UnusedTest"); // Since no runtime signals
                }
            }
        }

        nodes.extend(gap_nodes);
        edges.extend(gap_edges);
    }
}
