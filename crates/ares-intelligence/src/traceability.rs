use ares_knowledge_graph::store::KnowledgeGraphStore;
use ares_knowledge_graph::traversal::{MemoryTraversal, TraversalEngine};
use ares_store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracePath {
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityReport {
    pub root: String,
    pub traversal_depth: usize,
    pub requirements: Vec<String>,
    pub decisions: Vec<String>,
    pub files: Vec<String>,
    pub functions: Vec<String>,
    pub tests: Vec<String>,
    pub downstream: Vec<String>,
    pub upstream: Vec<String>,
    pub trace_paths: Vec<TracePath>,
    pub cycles_detected: bool,
    pub nodes_visited: usize,
    pub edges_traversed: usize,
    pub summary: String,
}

#[tracing::instrument(skip(store))]
pub async fn trace(
    entity_id: &str,
    depth: usize,
    store: &Store,
) -> anyhow::Result<TraceabilityReport> {
    let mut requirements = HashSet::new();
    let mut decisions = HashSet::new();
    let mut files = HashSet::new();
    let mut functions = HashSet::new();
    let mut tests = HashSet::new();
    let mut downstream = HashSet::new();
    let mut upstream = HashSet::new();
    let mut cycles_detected = false;
    let mut nodes_visited = 0;
    let mut edges_traversed = 0;
    let mut trace_paths = Vec::new();

    let kg_store = Arc::new(KnowledgeGraphStore::new(Arc::new(store.clone())));
    let traversal = TraversalEngine::new(kg_store);

    let down = traversal.downstream(entity_id, depth).unwrap();
    {
        let mut path_nodes = Vec::new();
        for node in &down.nodes {
            nodes_visited += 1;
            path_nodes.push(node.id.clone());
            if node.id != entity_id {
                downstream.insert(node.id.clone());
                let node_type_str = format!("{:?}", node.node_type);
                println!("DOWNSTREAM NODE: id={}, type={:?}", node.id, node_type_str);
                match node_type_str.as_str() {
                    "Requirement" => {
                        requirements.insert(node.id.clone());
                    }
                    "Decision" | "Architecture" => {
                        decisions.insert(node.id.clone());
                    }
                    "File" => {
                        files.insert(node.id.clone());
                    }
                    "Function" => {
                        functions.insert(node.id.clone());
                    }
                    "Test" => {
                        tests.insert(node.id.clone());
                    }
                    _ => {}
                }
            }
        }
        edges_traversed += down.edges.len();

        // Very basic path detection
        if !path_nodes.is_empty() {
            trace_paths.push(TracePath { nodes: path_nodes });
        }

        // Cycle detection - check if we saw fewer unique nodes than edges would imply
        if down.edges.len() >= down.nodes.len() && !down.nodes.is_empty() {
            cycles_detected = true;
        }
    }

    let up = traversal.upstream(entity_id, depth).unwrap();
    {
        let mut path_nodes = Vec::new();
        for node in &up.nodes {
            if node.id != entity_id {
                if !upstream.contains(&node.id) && !downstream.contains(&node.id) {
                    nodes_visited += 1;
                }
                upstream.insert(node.id.clone());
                let node_type_str = format!("{:?}", node.node_type);
                println!("UPSTREAM NODE: id={}, type={:?}", node.id, node_type_str);
                match node_type_str.as_str() {
                    "Requirement" => {
                        requirements.insert(node.id.clone());
                    }
                    "Decision" | "Architecture" => {
                        decisions.insert(node.id.clone());
                    }
                    "File" => {
                        files.insert(node.id.clone());
                    }
                    "Function" => {
                        functions.insert(node.id.clone());
                    }
                    "Test" => {
                        tests.insert(node.id.clone());
                    }
                    _ => {}
                }
            }
            path_nodes.push(node.id.clone());
        }
        edges_traversed += up.edges.len();

        if !path_nodes.is_empty() {
            path_nodes.reverse(); // Upstream path should end at entity
            trace_paths.push(TracePath { nodes: path_nodes });
        }

        if up.edges.len() >= up.nodes.len() && !up.nodes.is_empty() {
            cycles_detected = true;
        }
    }

    // Fallback if empty
    if nodes_visited == 0 {
        return Ok(TraceabilityReport {
            root: entity_id.to_string(),
            traversal_depth: depth,
            requirements: vec![],
            decisions: vec![],
            files: vec![],
            functions: vec![],
            tests: vec![],
            downstream: vec![],
            upstream: vec![],
            trace_paths: vec![],
            cycles_detected: false,
            nodes_visited: 0,
            edges_traversed: 0,
            summary: format!(
                "Entity '{}' has no known traceability relationships.",
                entity_id
            ),
        });
    }

    let summary = format!(
        "Requirement traces to {} decisions, {} files and {} tests. \
         Function impacts {} downstream callers.",
        decisions.len(),
        files.len(),
        tests.len(),
        downstream.len()
    );

    let mut requirements: Vec<_> = requirements.into_iter().collect();
    let mut decisions: Vec<_> = decisions.into_iter().collect();
    let mut files: Vec<_> = files.into_iter().collect();
    let mut functions: Vec<_> = functions.into_iter().collect();
    let mut tests: Vec<_> = tests.into_iter().collect();
    let mut downstream: Vec<_> = downstream.into_iter().collect();
    let mut upstream: Vec<_> = upstream.into_iter().collect();

    requirements.sort();
    decisions.sort();
    files.sort();
    functions.sort();
    tests.sort();
    downstream.sort();
    upstream.sort();

    Ok(TraceabilityReport {
        root: entity_id.to_string(),
        traversal_depth: depth,
        requirements,
        decisions,
        files,
        functions,
        tests,
        downstream,
        upstream,
        trace_paths,
        cycles_detected,
        nodes_visited,
        edges_traversed,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_store::Store;
    use tempfile::TempDir;

    fn setup_test_db() -> (Store, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = Store::open(&db_path).unwrap();

        let conn = store.get_conn().unwrap();
        conn.execute("INSERT INTO projects (id, name, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('p1', 'test', '/test', 'rust', 'domain', 'greenfield', 0, 0)", []).unwrap();

        // Nodes
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('req1', 'requirement', 'req1', '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('dec1', 'decision', 'dec1', '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('file1.rs', 'file', 'file1.rs', '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('func1', 'function', 'func1', '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('test1', 'test', 'test1', '{}', 0, 0)", []).unwrap();

        // Cycle nodes
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('c1', 'file', 'c1.rs', '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('c2', 'file', 'c2.rs', '{}', 0, 0)", []).unwrap();

        // Edges
        // req1 -> dec1 -> file1.rs -> func1 -> test1
        conn.execute("INSERT INTO graph_relationships (id, source_entity, target_entity, relationship_type, confidence_score, properties, created_at, updated_at) VALUES ('e1', 'req1', 'dec1', 'motivated_by', 1.0, '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_relationships (id, source_entity, target_entity, relationship_type, confidence_score, properties, created_at, updated_at) VALUES ('e2', 'dec1', 'file1.rs', 'motivated_by', 1.0, '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_relationships (id, source_entity, target_entity, relationship_type, confidence_score, properties, created_at, updated_at) VALUES ('e3', 'file1.rs', 'func1', 'contains', 1.0, '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_relationships (id, source_entity, target_entity, relationship_type, confidence_score, properties, created_at, updated_at) VALUES ('e4', 'func1', 'test1', 'validated_by', 1.0, '{}', 0, 0)", []).unwrap();

        // Cycle edges c1 -> c2 -> c1
        conn.execute("INSERT INTO graph_relationships (id, source_entity, target_entity, relationship_type, confidence_score, properties, created_at, updated_at) VALUES ('e5', 'c1', 'c2', 'depends_on', 1.0, '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_relationships (id, source_entity, target_entity, relationship_type, confidence_score, properties, created_at, updated_at) VALUES ('e6', 'c2', 'c1', 'depends_on', 1.0, '{}', 0, 0)", []).unwrap();

        (store, dir)
    }

    #[tokio::test]
    async fn test_requirement_traversal() {
        let (store, _dir) = setup_test_db();
        let report = trace("req1", 3, &store).await.unwrap();
        println!("REPORT: {:#?}", report);
        assert_eq!(report.root, "req1");
        assert!(report.decisions.contains(&"dec1".to_string()));
        assert!(report.files.contains(&"file1.rs".to_string()));
    }

    #[tokio::test]
    async fn test_function_traversal() {
        let (store, _dir) = setup_test_db();
        let report = trace("func1", 3, &store).await.unwrap();
        assert!(report.tests.contains(&"test1".to_string()));
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let (store, _dir) = setup_test_db();
        let report = trace("c1", 3, &store).await.unwrap();
        assert!(report.cycles_detected);
    }

    #[tokio::test]
    async fn test_depth_limiting() {
        let (store, _dir) = setup_test_db();
        let r1 = trace("req1", 1, &store).await.unwrap();
        assert!(r1.decisions.contains(&"dec1".to_string()));
        assert!(!r1.files.contains(&"file1.rs".to_string())); // Beyond depth 1

        let r2 = trace("req1", 2, &store).await.unwrap();
        assert!(r2.files.contains(&"file1.rs".to_string()));
    }

    #[tokio::test]
    async fn test_missing_node() {
        let (store, _dir) = setup_test_db();
        let report = trace("unknown_node", 3, &store).await.unwrap();
        assert_eq!(report.nodes_visited, 0);
        assert!(!report.summary.is_empty());
    }
}
