use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use super::service::SearchService;
use super::super::entities::models::Entity;

#[derive(Clone)]
pub struct SearchApiState {
    pub service: Arc<SearchService>,
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<Entity>,
}

pub fn router<S>(state: SearchApiState) -> Router<S> 
where 
    S: Clone + Send + Sync + 'static
{
    Router::new()
        .route("/", post(search))
        .with_state(state)
}

async fn search(
    State(state): State<SearchApiState>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, String> {
    let results = state.service.search(&payload.query).await.map_err(|e| e.to_string())?;
    Ok(Json(SearchResponse { results }))
}
