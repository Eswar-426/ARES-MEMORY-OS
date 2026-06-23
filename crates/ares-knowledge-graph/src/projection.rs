use crate::models::{
    DomainEvent, DomainEventType, EdgeType, KnowledgeEdge, KnowledgeNode, ProjectionMetrics,
};
use crate::store::KnowledgeGraphStore;
use ares_core::AresError;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum ProjectionMode {
    FullRebuild,
    Incremental,
}

pub struct ProjectionBatch {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

impl Default for ProjectionBatch {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectionBatch {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

pub trait GraphProjector: Send + Sync {
    fn supports(&self, event_type: &DomainEventType) -> bool;
    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError>;
}

pub fn generate_edge_id(source_id: &str, edge_type: &EdgeType, target_id: &str) -> String {
    format!("{}:{}:{}", source_id, edge_type, target_id)
}

pub struct ProjectionEngine {
    store: Arc<KnowledgeGraphStore>,
}

impl ProjectionEngine {
    pub fn new(store: Arc<KnowledgeGraphStore>) -> Self {
        Self { store }
    }

    pub fn process_event(
        &self,
        event: &DomainEvent,
        mode: ProjectionMode,
        projectors: &[Box<dyn GraphProjector>],
        metrics: &mut ProjectionMetrics,
    ) -> Result<(), AresError> {
        metrics.events_processed += 1;

        match mode {
            ProjectionMode::Incremental => {
                if self.store.is_event_projected(&event.id)? {
                    metrics.duplicate_events_skipped += 1;
                    return Ok(());
                }
            }
            ProjectionMode::FullRebuild => {}
        }

        let mut projected_anything = false;

        for projector in projectors {
            if !projector.supports(&event.event_type) {
                continue;
            }

            projected_anything = true;
            let batch = match projector.project(event) {
                Ok(b) => b,
                Err(e) => {
                    metrics.projection_failures += 1;
                    return Err(e);
                }
            };

            for node in batch.nodes {
                self.store.upsert_node(&node)?;
                metrics.nodes_created += 1;
            }

            for mut edge in batch.edges {
                edge.id = generate_edge_id(&edge.source_id, &edge.edge_type, &edge.target_id);
                self.store.upsert_edge(&edge)?;
                metrics.edges_created += 1;
            }
        }

        if projected_anything {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            self.store.mark_event_projected(&event.id, now)?;
        }

        Ok(())
    }
}
