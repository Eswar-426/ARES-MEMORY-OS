use axum::{extract::{State, Path}, Json};
use std::sync::Arc;
use crate::models::{ApiResponse, ApiErrorEnvelope};
use ares_memory_intelligence::facade::MemoryFacade;

#[utoipa::path(
    get,
    path = "/api/v1/memory/why/{id}",
    responses((status = 200, description = "Why entity exists", body = ApiResponse<serde_json::Value>))
)]
pub async fn why(
    State(facade): State<Arc<MemoryFacade>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, Json<ApiErrorEnvelope>> {
    match facade.why(&id) {
        Ok(res) => Ok(Json(ApiResponse::success(res))),
        Err(e) => Err(Json(ApiErrorEnvelope::new("INTERNAL_ERROR", &e.to_string()))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/memory/who/{id}",
    responses((status = 200, description = "Who owns entity", body = ApiResponse<serde_json::Value>))
)]
pub async fn who(
    State(facade): State<Arc<MemoryFacade>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, Json<ApiErrorEnvelope>> {
    match facade.who(&id) {
        Ok(res) => Ok(Json(ApiResponse::success(res))),
        Err(e) => Err(Json(ApiErrorEnvelope::new("INTERNAL_ERROR", &e.to_string()))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/memory/impact/{id}",
    responses((status = 200, description = "Impact analysis", body = ApiResponse<serde_json::Value>))
)]
pub async fn impact(
    State(facade): State<Arc<MemoryFacade>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, Json<ApiErrorEnvelope>> {
    match facade.impact(&id) {
        Ok(res) => {
            let val = serde_json::to_value(res).unwrap_or(serde_json::json!({}));
            Ok(Json(ApiResponse::success(val)))
        },
        Err(e) => Err(Json(ApiErrorEnvelope::new("INTERNAL_ERROR", &e.to_string()))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/memory/evolution/{id}",
    responses((status = 200, description = "Evolution timeline", body = ApiResponse<serde_json::Value>))
)]
pub async fn evolution(
    State(facade): State<Arc<MemoryFacade>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, Json<ApiErrorEnvelope>> {
    match facade.evolution(&id) {
        Ok(res) => {
            let val = serde_json::to_value(res).unwrap_or(serde_json::json!({}));
            Ok(Json(ApiResponse::success(val)))
        },
        Err(e) => Err(Json(ApiErrorEnvelope::new("INTERNAL_ERROR", &e.to_string()))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/memory/facade_context/{id}",
    responses((status = 200, description = "Memory context package", body = ApiResponse<serde_json::Value>))
)]
pub async fn context(
    State(facade): State<Arc<MemoryFacade>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, Json<ApiErrorEnvelope>> {
    match facade.context(&id) {
        Ok(res) => Ok(Json(ApiResponse::success(res))),
        Err(e) => Err(Json(ApiErrorEnvelope::new("INTERNAL_ERROR", &e.to_string()))),
    }
}
