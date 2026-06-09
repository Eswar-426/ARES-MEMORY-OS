use super::models::ContextGraph;
use crate::entities::repository::EntityRepository;
use crate::graph::traversal::engine::{TraversalEngine, TraversalStrategy};
use crate::relationships::repository::RelationshipRepository;
use ares_core::AresError;
use ares_store::db::Store;
use uuid::Uuid;

pub struct ContextBuilderService {
    db: Store,
    entity_repo: EntityRepository,
    _relationship_repo: RelationshipRepository,
    traversal_engine: TraversalEngine,
}

impl ContextBuilderService {
    pub fn new(db: Store) -> Self {
        Self {
            db,
            entity_repo: EntityRepository::new(),
            _relationship_repo: RelationshipRepository::new(),
            traversal_engine: TraversalEngine::new(),
        }
    }

    pub async fn build_context(
        &self,
        focal_entity_id: Uuid,
        max_depth: u32,
    ) -> Result<ContextGraph, AresError> {
        let conn = self.db.get_conn()?;

        let path =
            self.traversal_engine
                .traverse(focal_entity_id, TraversalStrategy::BFS, max_depth);

        let mut entities = Vec::new();
        for id in path {
            if let Some(entity) = self.entity_repo.get_by_id(&conn, id)? {
                entities.push(entity);
            }
        }

        // Mock pulling relationships for the context graph
        let relationships = Vec::new();

        Ok(ContextGraph {
            focal_entity_id,
            entities,
            relationships,
            depth: max_depth,
        })
    }
}
