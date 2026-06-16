use ares_core::AresError;
use ares_store::Store;
use std::sync::Arc;
use crate::models::{KnowledgeNode, KnowledgeEdge, NodeType, EdgeType};
use rusqlite::params;

pub struct KnowledgeGraphStore {
    store: Arc<Store>,
}

impl KnowledgeGraphStore {
    pub fn new(store: Arc<ares_store::Store>) -> Self {
        Self { store }
    }

    pub(crate) fn get_raw_store(&self) -> Arc<ares_store::Store> {
        self.store.clone()
    }

    /// Upserts a KnowledgeNode into graph_entities. 
    /// This is strictly idempotent because it uses INSERT OR REPLACE on the identical node ID.
    pub fn upsert_node(&self, node: &KnowledgeNode) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        
        let props_str = serde_json::to_string(&node.properties)
            .map_err(|e| AresError::Serialization(format!("Failed to serialize properties: {}", e)))?;
            
        conn.execute(
            "INSERT OR REPLACE INTO graph_entities (
                id, entity_type, name, properties, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![
                node.id,
                node.node_type.to_string(),
                node.name,
                props_str,
                node.created_at.to_string()
            ],
        ).map_err(|e| AresError::Database(e.to_string()))?;

        Ok(())
    }

    /// Upserts a KnowledgeEdge into graph_relationships.
    /// This is strictly idempotent assuming edge IDs are deterministic.
    pub fn upsert_edge(&self, edge: &KnowledgeEdge) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;

        let props_str = serde_json::to_string(&edge.properties)
            .map_err(|e| AresError::Serialization(format!("Failed to serialize edge properties: {}", e)))?;

        conn.execute(
            "INSERT OR REPLACE INTO graph_relationships (
                id, source_entity, target_entity, relationship_type, 
                confidence_score, properties, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                edge.id,
                edge.source_id,
                edge.target_id,
                edge.edge_type.to_string(),
                edge.confidence,
                props_str,
                edge.created_at.to_string()
            ],
        ).map_err(|e| AresError::Database(e.to_string()))?;

        Ok(())
    }

    /// Checks if an event has already been projected.
    pub fn is_event_projected(&self, event_id: &str) -> Result<bool, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare("SELECT 1 FROM knowledge_events WHERE id = ?1 AND status = 'PROJECTED'")
            .map_err(|e| AresError::Database(e.to_string()))?;
        let exists = stmt.exists(params![event_id])
            .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(exists)
    }

    /// Marks an event as projected in the ledger.
    pub fn mark_event_projected(&self, event_id: &str, projected_at: i64) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO knowledge_events (id, event_type, payload, processed_at, status) 
             VALUES (?1, 'PROJECTION', '{}', ?2, 'PROJECTED')",
            params![event_id, projected_at.to_string()]
        ).map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }
    
    // For testing purposes
    pub fn count_nodes(&self) -> Result<i64, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM graph_entities")
            .map_err(|e| AresError::Database(e.to_string()))?;
        let count: i64 = stmt.query_row([], |row| row.get(0))
            .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(count)
    }

    pub fn count_edges(&self) -> Result<i64, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM graph_relationships")
            .map_err(|e| AresError::Database(e.to_string()))?;
        let count: i64 = stmt.query_row([], |row| row.get(0))
            .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(count)
    }
}
