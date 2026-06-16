use crate::models::{KnowledgeEdge, KnowledgeNode, EdgeType};
use crate::store::KnowledgeGraphStore;
use ares_core::AresError;
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct TraversalPath {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

pub trait MemoryTraversal {
    fn upstream(&self, start_node_id: &str, max_depth: usize) -> Result<TraversalPath, AresError>;
    fn downstream(&self, start_node_id: &str, max_depth: usize) -> Result<TraversalPath, AresError>;
    fn shortest_path(&self, start_node_id: &str, target_node_id: &str) -> Result<Option<TraversalPath>, AresError>;
}

pub struct TraversalEngine {
    store: Arc<KnowledgeGraphStore>,
}

impl TraversalEngine {
    pub fn new(store: Arc<KnowledgeGraphStore>) -> Self {
        Self { store }
    }

    fn load_node(&self, id: &str) -> Result<Option<KnowledgeNode>, AresError> {
        let conn = self.store.get_raw_store().get_conn()?;
        let mut stmt = conn.prepare("SELECT id, entity_type, name, properties, created_at FROM graph_entities WHERE id = ?")
            .map_err(|e| AresError::Database(e.to_string()))?;
            
        let mut rows = stmt.query([id]).map_err(|e| AresError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AresError::Database(e.to_string()))? {
            let id: String = row.get(0).map_err(|e| AresError::Database(e.to_string()))?;
            let node_type_str: String = row.get(1).map_err(|e| AresError::Database(e.to_string()))?;
            let name: String = row.get(2).map_err(|e| AresError::Database(e.to_string()))?;
            let props_str: String = row.get(3).map_err(|e| AresError::Database(e.to_string()))?;
            let created_at_str: String = row.get(4).map_err(|e| AresError::Database(e.to_string()))?;
            let created_at: i64 = created_at_str.parse().unwrap_or(0);
            
            let properties: serde_json::Value = serde_json::from_str(&props_str).unwrap_or(serde_json::json!({}));
            
            let node_type = serde_json::from_value(serde_json::json!(node_type_str.clone())).unwrap_or_else(|_| {
                crate::models::NodeType::CodeArtifact
            });

            return Ok(Some(KnowledgeNode {
                id,
                node_type,
                name,
                properties,
                created_at,
            }));
        }
        Ok(None)
    }

    fn load_adjacent_edges(&self, node_id: &str, direction_downstream: bool) -> Result<Vec<KnowledgeEdge>, AresError> {
        let conn = self.store.get_raw_store().get_conn()?;
        let query = if direction_downstream {
            "SELECT id, source_entity, target_entity, relationship_type, confidence_score, created_at, properties FROM graph_relationships WHERE source_entity = ?"
        } else {
            "SELECT id, source_entity, target_entity, relationship_type, confidence_score, created_at, properties FROM graph_relationships WHERE target_entity = ?"
        };
        let mut stmt = conn.prepare(query).map_err(|e| AresError::Database(e.to_string()))?;
        
        let mut edges = Vec::new();
        let mut rows = stmt.query([node_id]).map_err(|e| AresError::Database(e.to_string()))?;
        while let Some(row) = rows.next().map_err(|e| AresError::Database(e.to_string()))? {
            let id: String = row.get(0).map_err(|e| AresError::Database(e.to_string()))?;
            let source_id: String = row.get(1).map_err(|e| AresError::Database(e.to_string()))?;
            let target_id: String = row.get(2).map_err(|e| AresError::Database(e.to_string()))?;
            let edge_type_str: String = row.get(3).map_err(|e| AresError::Database(e.to_string()))?;
            let confidence: f64 = row.get(4).unwrap_or(1.0);
            let created_at_str: String = row.get(5).unwrap_or_else(|_| "0".to_string());
            let created_at = created_at_str.parse().unwrap_or(0);
            let props_str: String = row.get(6).unwrap_or("{}".to_string());
            
            let properties: serde_json::Value = serde_json::from_str(&props_str).unwrap_or(serde_json::json!({}));
            let edge_type = serde_json::from_value(serde_json::json!(edge_type_str)).unwrap_or(crate::models::EdgeType::References);

            edges.push(KnowledgeEdge {
                id,
                source_id,
                target_id,
                edge_type,
                confidence: confidence as f32,
                created_at,
                properties,
            });
        }
        Ok(edges)
    }

    fn traverse(
        &self,
        start_node_id: &str,
        max_depth: usize,
        direction_downstream: bool,
    ) -> Result<TraversalPath, AresError> {
        let mut visited_nodes = HashSet::new();
        let mut visited_edges = HashSet::new();
        let mut result_nodes = Vec::new();
        let mut result_edges = Vec::new();

        let mut queue = VecDeque::new();
        queue.push_back((start_node_id.to_string(), 0));

        while let Some((curr_node, depth)) = queue.pop_front() {
            if visited_nodes.contains(&curr_node) {
                continue;
            }
            visited_nodes.insert(curr_node.clone());

            if let Some(node) = self.load_node(&curr_node)? {
                result_nodes.push(node);
            }

            if depth < max_depth {
                let edges = self.load_adjacent_edges(&curr_node, direction_downstream)?;
                for edge in edges {
                    let next_node = if direction_downstream {
                        &edge.target_id
                    } else {
                        &edge.source_id
                    };

                    if !visited_edges.contains(&edge.id) {
                        visited_edges.insert(edge.id.clone());
                        result_edges.push(edge.clone());
                    }

                    if !visited_nodes.contains(next_node) {
                        queue.push_back((next_node.clone(), depth + 1));
                    }
                }
            }
        }

        Ok(TraversalPath {
            nodes: result_nodes,
            edges: result_edges,
        })
    }
}

impl MemoryTraversal for TraversalEngine {
    fn upstream(&self, start_node_id: &str, max_depth: usize) -> Result<TraversalPath, AresError> {
        self.traverse(start_node_id, max_depth, false)
    }

    fn downstream(&self, start_node_id: &str, max_depth: usize) -> Result<TraversalPath, AresError> {
        self.traverse(start_node_id, max_depth, true)
    }

    fn shortest_path(&self, start_node_id: &str, target_node_id: &str) -> Result<Option<TraversalPath>, AresError> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent_map: HashMap<String, (String, KnowledgeEdge)> = HashMap::new();

        queue.push_back(start_node_id.to_string());
        visited.insert(start_node_id.to_string());

        let mut found = false;

        while let Some(curr) = queue.pop_front() {
            if curr == target_node_id {
                found = true;
                break;
            }

            let edges = self.load_adjacent_edges(&curr, true)?;
            for edge in edges {
                if !visited.contains(&edge.target_id) {
                    visited.insert(edge.target_id.clone());
                    parent_map.insert(edge.target_id.clone(), (curr.clone(), edge.clone()));
                    queue.push_back(edge.target_id.clone());
                }
            }
        }

        if !found {
            return Ok(None);
        }

        let mut result_nodes = Vec::new();
        let mut result_edges = Vec::new();
        let mut curr = target_node_id.to_string();

        if let Some(node) = self.load_node(&curr)? {
            result_nodes.push(node);
        }

        while let Some((prev, edge)) = parent_map.get(&curr) {
            result_edges.push(edge.clone());
            if let Some(node) = self.load_node(prev)? {
                result_nodes.push(node);
            }
            curr = prev.clone();
        }

        result_nodes.reverse();
        result_edges.reverse();

        Ok(Some(TraversalPath {
            nodes: result_nodes,
            edges: result_edges,
        }))
    }
}
