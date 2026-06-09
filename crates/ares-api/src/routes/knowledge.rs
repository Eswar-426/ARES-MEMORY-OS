use ares_knowledge::entities::api::{router as entities_router, EntityApiState};
use ares_knowledge::relationships::api::{router as relationships_router, RelationshipApiState};
use ares_knowledge::search::api::{router as search_router, SearchApiState};
use ares_knowledge::entities::service::EntityService;
use ares_knowledge::relationships::service::RelationshipService;
use ares_knowledge::search::service::SearchService;
use axum::Router;
use std::sync::Arc;
use ares_store::db::Store;

pub fn router<S>(db: Store) -> Router<S> 
where 
    S: Clone + Send + Sync + 'static
{
    let entity_service = Arc::new(EntityService::new(db.clone()));
    let relationship_service = Arc::new(RelationshipService::new(db.clone()));
    let search_service = Arc::new(SearchService::new(db.clone()));

    let entities = entities_router(EntityApiState { service: entity_service });
    let relationships = relationships_router(RelationshipApiState { service: relationship_service });
    let search = search_router(SearchApiState { service: search_service });

    Router::new()
        .nest("/entities", entities)
        .nest("/relationships", relationships)
        .nest("/search", search)
}
