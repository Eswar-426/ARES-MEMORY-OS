use super::models::Relationship;
use super::service::RelationshipService;
use axum::{extract::State, routing::post, Json, Router};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct RelationshipApiState {
    pub service: Arc<RelationshipService>,
}

#[derive(Deserialize)]
pub struct CreateRelationshipRequest {
    pub source_entity: Uuid,
    pub target_entity: Uuid,
    pub relationship_type: String,
    pub properties: serde_json::Value,
}

pub fn router<S>(state: RelationshipApiState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/", post(create_relationship))
        .with_state(state)
}

async fn create_relationship(
    State(state): State<RelationshipApiState>,
    Json(payload): Json<CreateRelationshipRequest>,
) -> Result<Json<Relationship>, String> {
    let rel = Relationship {
        id: Uuid::now_v7(),
        source_entity: payload.source_entity,
        target_entity: payload.target_entity,
        relationship_type: payload.relationship_type,
        properties: payload.properties,
        embedding: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        valid_from: None,
        valid_to: None,
        confidence_score: 1.0,
        evidence_count: 1,
        source_event_id: None,
    };

    let created = state
        .service
        .create_relationship(rel)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(created))
}
