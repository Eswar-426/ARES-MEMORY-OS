use async_trait::async_trait;
use rusqlite::params;

use crate::db::Store;
use ares_core::id::ProjectId;
use ares_core::types::evidence::Evidence;
use ares_core::{AresError, GraphNode, NodeType};

#[async_trait]
pub trait EvidenceRepository: Send + Sync {
    async fn record_evidence(&self, project_id: &str, evidence: Evidence) -> Result<(), AresError>;
    async fn get_evidence_for_node(
        &self,
        project_id: &str,
        node_id: &str,
    ) -> Result<Vec<Evidence>, AresError>;
}

pub struct SqliteEvidenceRepository {
    store: Store,
}

impl SqliteEvidenceRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

#[async_trait]
impl EvidenceRepository for SqliteEvidenceRepository {
    async fn record_evidence(&self, project_id: &str, evidence: Evidence) -> Result<(), AresError> {
        let mut conn = self.store.get_conn()?;
        let tx = conn.transaction().map_err(AresError::db)?;

        // 1. Insert Evidence as GraphNode
        let properties =
            serde_json::to_value(&evidence).map_err(|e| AresError::Serialization(e.to_string()))?;
        let node = GraphNode {
            id: evidence.id.clone(),
            project_id: ProjectId::from(project_id.to_string()),
            node_type: NodeType::Evidence,
            label: format!("{:?}", evidence.evidence_type),
            properties,
            file_path: None,
            created_at: evidence.observed_at.timestamp(),
            updated_at: evidence.observed_at.timestamp(),
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
        .map_err(|e| AresError::db(format!("Failed to insert Evidence GraphNode: {}", e)))?;

        // 2. Insert Edge from Evidence to the source node (the node it observes or contradicts or supports)
        // By default, the evidence "observes" the node it came from
        tx.execute(
            "INSERT INTO graph_edges (id, project_id, from_node_id, to_node_id, edge_type, weight, confidence, source, valid_from, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                ares_core::id::new_id(),
                project_id,
                evidence.id.as_str(),
                evidence.source_node.as_str(),
                "observes",
                1.0,
                evidence.confidence,
                "scanner", // TODO: adapt based on evidence.source
                evidence.observed_at.timestamp(),
                evidence.observed_at.timestamp(),
            ],
        )
        .map_err(|e| AresError::db(format!("Failed to insert Edge: {}", e)))?;

        tx.commit().map_err(AresError::db)?;
        Ok(())
    }

    async fn get_evidence_for_node(
        &self,
        project_id: &str,
        node_id: &str,
    ) -> Result<Vec<Evidence>, AresError> {
        let conn = self.store.get_conn()?;

        let mut stmt = conn
            .prepare(
                "SELECT n.properties 
                 FROM graph_nodes n
                 JOIN graph_edges e ON n.id = e.from_node_id
                 WHERE n.project_id = ?1 
                   AND e.to_node_id = ?2 
                   AND n.node_type = 'evidence'
                 ORDER BY n.created_at DESC",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id, node_id], |row| {
                let props_str: String = row.get(0)?;
                let event: Evidence =
                    serde_json::from_str(&props_str).map_err(|_| rusqlite::Error::InvalidQuery)?;
                Ok(event)
            })
            .map_err(AresError::db)?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r.map_err(AresError::db)?);
        }

        Ok(events)
    }
}
