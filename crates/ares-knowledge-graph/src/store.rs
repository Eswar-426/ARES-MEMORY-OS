use crate::models::{EdgeType, KnowledgeEdge, KnowledgeNode, NodeType};
use ares_core::AresError;
use ares_store::Store;
use rusqlite::params;
use std::sync::Arc;

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

        let props_str = serde_json::to_string(&node.properties).map_err(|e| {
            AresError::Serialization(format!("Failed to serialize properties: {}", e))
        })?;

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
        )
        .map_err(|e| AresError::Database(e.to_string()))?;

        Ok(())
    }

    /// Upserts a KnowledgeEdge into graph_relationships.
    /// This is strictly idempotent assuming edge IDs are deterministic.
    pub fn upsert_edge(&self, edge: &KnowledgeEdge) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;

        let props_str = serde_json::to_string(&edge.properties).map_err(|e| {
            AresError::Serialization(format!("Failed to serialize edge properties: {}", e))
        })?;

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
        )
        .map_err(|e| {
            eprintln!(
                "Failed to upsert edge: {:?} -> {:?}, Error: {}",
                edge.source_id, edge.target_id, e
            );
            AresError::Database(e.to_string())
        })?;

        Ok(())
    }

    pub fn upsert_batch(&self, events: &[crate::models::GraphEvent]) -> Result<(), AresError> {
        let mut conn = self.store.get_conn()?;
        let tx = conn
            .transaction()
            .map_err(|e| AresError::Database(e.to_string()))?;

        {
            let mut node_stmt = tx
                .prepare_cached(
                    "INSERT OR REPLACE INTO graph_entities (
                    id, entity_type, name, properties, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                )
                .map_err(|e| AresError::Database(e.to_string()))?;

            let mut edge_stmt = tx
                .prepare_cached(
                    "INSERT OR REPLACE INTO graph_relationships (
                    id, source_entity, target_entity, relationship_type, 
                    confidence_score, properties, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                )
                .map_err(|e| AresError::Database(e.to_string()))?;

            for event in events {
                match event {
                    crate::models::GraphEvent::Node(node) => {
                        let props_str = serde_json::to_string(&node.properties)
                            .unwrap_or_else(|_| "{}".to_string());
                        node_stmt
                            .execute(params![
                                node.id,
                                node.node_type.to_string(),
                                node.name,
                                props_str,
                                node.created_at.to_string()
                            ])
                            .map_err(|e| {
                                AresError::Database(format!("Node Insert Error: {}", e))
                            })?;
                    }
                    crate::models::GraphEvent::Edge(edge) => {
                        let props_str = serde_json::to_string(&edge.properties)
                            .unwrap_or_else(|_| "{}".to_string());
                        edge_stmt
                            .execute(params![
                                edge.id,
                                edge.source_id,
                                edge.target_id,
                                edge.edge_type.to_string(),
                                edge.confidence,
                                props_str,
                                edge.created_at.to_string()
                            ])
                            .map_err(|_e| {
                                AresError::Database(format!(
                                    "Edge Insert Error: {} -> {}",
                                    edge.source_id, edge.target_id
                                ))
                            })?;
                    }
                }
            }
        }

        tx.commit()
            .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }

    /// Checks if an event has already been projected.
    pub fn is_event_projected(&self, event_id: &str) -> Result<bool, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT 1 FROM knowledge_events WHERE id = ?1 AND status = 'PROJECTED'")
            .map_err(|e| AresError::Database(e.to_string()))?;
        let exists = stmt
            .exists(params![event_id])
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
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM graph_entities")
            .map_err(|e| AresError::Database(e.to_string()))?;
        let count: i64 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(count)
    }

    pub fn count_edges(&self) -> Result<i64, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM graph_relationships")
            .map_err(|e| AresError::Database(e.to_string()))?;
        let count: i64 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(count)
    }

    /// Exports the entire graph (nodes and edges) for delta computation or snapshotting.
    pub fn export_graph(&self) -> Result<crate::models::KnowledgeGraph, AresError> {
        let conn = self.store.get_conn()?;

        let mut node_stmt = conn
            .prepare("SELECT id, entity_type, name, properties, created_at FROM graph_entities")
            .map_err(|e| AresError::Database(e.to_string()))?;

        let node_iter = node_stmt
            .query_map([], |row| {
                let n_type: String = row.get(1)?;
                let props_str: String = row.get(3)?;
                let created_str: String = row.get(4)?;

                let node_type = match n_type.as_str() {
                    "Requirement" => NodeType::Requirement,
                    "RequirementRevision" => NodeType::RequirementRevision,
                    "Decision" => NodeType::Decision,
                    "DecisionRevision" => NodeType::DecisionRevision,
                    "Evidence" => NodeType::Evidence,
                    "Outcome" => NodeType::Outcome,
                    "Architecture" => NodeType::Architecture,
                    "CodeArtifact" => NodeType::CodeArtifact,
                    "Test" => NodeType::Test,
                    "RuntimeSignal" => NodeType::RuntimeSignal,
                    "Gap" => NodeType::Gap,
                    "RootCause" => NodeType::RootCause,
                    "Resolution" => NodeType::Resolution,
                    "Owner" => NodeType::Owner,
                    "Repository" => NodeType::Repository,
                    "Project" => NodeType::Project,
                    "RepositoryEvent" => NodeType::RepositoryEvent,
                    "RepositorySnapshot" => NodeType::RepositorySnapshot,
                    "KnowledgeGap" => NodeType::KnowledgeGap,
                    _ => NodeType::CodeArtifact, // fallback
                };

                let properties =
                    serde_json::from_str(&props_str).unwrap_or(serde_json::Value::Null);
                let created_at = created_str.parse::<i64>().unwrap_or(0);

                Ok(KnowledgeNode {
                    id: row.get(0)?,
                    node_type,
                    name: row.get(2)?,
                    properties,
                    created_at,
                })
            })
            .map_err(|e| AresError::Database(e.to_string()))?;

        let mut nodes = Vec::new();
        for node in node_iter.flatten() {
            nodes.push(node);
        }

        let mut edge_stmt = conn.prepare("SELECT id, source_entity, target_entity, relationship_type, confidence_score, created_at, properties FROM graph_relationships")
            .map_err(|e| AresError::Database(e.to_string()))?;

        let edge_iter = edge_stmt
            .query_map([], |row| {
                let e_type: String = row.get(3)?;
                let confidence: f64 = row.get(4)?;
                let created_str: String = row.get(5)?;
                let props_str: String = row.get(6)?;

                let edge_type = match e_type.as_str() {
                    "Implements" => EdgeType::Implements,
                    "ImplementedBy" => EdgeType::ImplementedBy,
                    "Drives" => EdgeType::Drives,
                    "DependsOn" => EdgeType::DependsOn,
                    "SupportedBy" => EdgeType::SupportedBy,
                    "Supports" => EdgeType::Supports,
                    "ValidatedBy" => EdgeType::ValidatedBy,
                    "ResultsIn" => EdgeType::ResultsIn,
                    "OwnedBy" => EdgeType::OwnedBy,
                    "Exhibits" => EdgeType::Exhibits,
                    "Causes" => EdgeType::Causes,
                    "Resolves" => EdgeType::Resolves,
                    "References" => EdgeType::References,
                    "TracesTo" => EdgeType::TracesTo,
                    "ApprovedBy" => EdgeType::ApprovedBy,
                    "DerivedFrom" => EdgeType::DerivedFrom,
                    "Supersedes" => EdgeType::Supersedes,
                    "Contains" => EdgeType::Contains,
                    "OccurredIn" => EdgeType::OccurredIn,
                    "GeneratedFrom" => EdgeType::GeneratedFrom,
                    "HasGap" => EdgeType::HasGap,
                    _ => EdgeType::References, // fallback
                };

                let properties =
                    serde_json::from_str(&props_str).unwrap_or(serde_json::Value::Null);
                let created_at = created_str.parse::<i64>().unwrap_or(0);

                Ok(KnowledgeEdge {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    edge_type,
                    confidence: confidence as f32,
                    created_at,
                    properties,
                })
            })
            .map_err(|e| AresError::Database(e.to_string()))?;

        let mut edges = Vec::new();
        for edge in edge_iter.flatten() {
            edges.push(edge);
        }

        Ok(crate::models::KnowledgeGraph { nodes, edges })
    }
}
