use async_trait::async_trait;
use rusqlite::params;

use crate::db::Store;
use ares_core::id::ProjectId;
use ares_core::types::evolution::EvolutionEvent;
use ares_core::{EdgeType, GraphNode, NodeType};

#[async_trait]
pub trait EvolutionRepository: Send + Sync {
    async fn record_event(&self, project_id: &str, event: &EvolutionEvent) -> Result<(), String>;
    async fn get_events_for_node(
        &self,
        project_id: &str,
        target_node: &str,
    ) -> Result<Vec<EvolutionEvent>, String>;
}

pub struct SqliteEvolutionRepository {
    store: Store,
}

impl SqliteEvolutionRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

#[async_trait]
impl EvolutionRepository for SqliteEvolutionRepository {
    async fn record_event(&self, project_id: &str, event: &EvolutionEvent) -> Result<(), String> {
        let mut conn = self.store.get_conn().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // 1. Insert the EvolutionEvent as a GraphNode
        let properties = serde_json::to_value(event).map_err(|e| e.to_string())?;
        let node = GraphNode {
            id: event.id.clone(),
            project_id: ProjectId::from(project_id.to_string()),
            node_type: NodeType::EvolutionEvent,
            label: format!("{:?}", event.event_type),
            properties,
            file_path: None,
            created_at: event.occurred_at,
            updated_at: event.occurred_at,
            deleted_at: None,
        };

        tx.execute(
            "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, file_path, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                node.id.as_str(),
                node.project_id.as_str(),
                node.node_type.as_str(),
                node.label,
                node.properties.to_string(),
                node.file_path,
                node.created_at,
                node.updated_at,
            ],
        )
        .map_err(|e| format!("Failed to insert EvolutionEvent GraphNode: {}", e))?;

        // 2. Insert the edge from EvolutionEvent to the target node
        tx.execute(
            "INSERT INTO graph_edges (id, project_id, from_node_id, to_node_id, edge_type, weight, confidence, source, valid_from, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                ares_core::id::new_id(),
                project_id,
                event.id.as_str(),
                event.target_node.as_str(),
                EdgeType::Evolves.as_str(),
                1.0,
                event.confidence,
                "evolution_engine",
                event.occurred_at,
                event.occurred_at,
            ],
        )
        .map_err(|e| format!("Failed to insert Evolves Edge: {}", e))?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_events_for_node(
        &self,
        project_id: &str,
        target_node: &str,
    ) -> Result<Vec<EvolutionEvent>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT n.properties 
                 FROM graph_nodes n
                 JOIN graph_edges e ON n.id = e.from_node_id
                 WHERE n.project_id = ?1 
                   AND e.to_node_id = ?2 
                   AND n.node_type = 'evolution_event'
                   AND e.edge_type = 'evolves'
                 ORDER BY n.created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![project_id, target_node], |row| {
                let props_str: String = row.get(0)?;
                let event: EvolutionEvent =
                    serde_json::from_str(&props_str).map_err(|_| rusqlite::Error::InvalidQuery)?;
                Ok(event)
            })
            .map_err(|e| e.to_string())?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r.map_err(|e| e.to_string())?);
        }

        Ok(events)
    }
}
