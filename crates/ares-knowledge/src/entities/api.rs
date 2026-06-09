use super::models::Entity;
use super::service::EntityService;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct EntityApiState {
    pub service: Arc<EntityService>,
}

#[derive(Deserialize)]
pub struct CreateEntityRequest {
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub properties: serde_json::Value,
}

pub fn router<S>(state: EntityApiState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/", post(create_entity))
        .route("/:id", get(get_entity))
        .with_state(state)
}

async fn create_entity(
    State(state): State<EntityApiState>,
    Json(payload): Json<CreateEntityRequest>,
) -> Result<Json<Entity>, String> {
    let entity = Entity {
        id: Uuid::now_v7(),
        entity_type: payload.entity_type,
        name: payload.name,
        description: payload.description,
        properties: payload.properties,
        embedding: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        valid_from: None,
        valid_to: None,
        confidence_score: 1.0,
        source_event_id: None,
    };

    let created = state
        .service
        .create_entity(entity)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(created))
}

async fn get_entity(
    State(state): State<EntityApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Entity>, String> {
    let entity = state
        .service
        .get_entity(id)
        .await
        .map_err(|e| e.to_string())?;
    match entity {
        Some(e) => Ok(Json(e)),
        None => Err("Entity not found".to_string()),
    }
}
