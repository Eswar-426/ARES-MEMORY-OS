use axum::{extract::State, Json};
use std::sync::Arc;
use crate::models::{ApiResponse, ApiErrorEnvelope, HealthStatus};
use ares_validation::validation_runner::ValidationRunner;

pub async fn health() -> Result<Json<ApiResponse<HealthStatus>>, Json<ApiErrorEnvelope>> {
    Ok(Json(ApiResponse::success(HealthStatus {
        status: "UP".to_string(),
        version: "v0.8.0-memory-api".to_string(),
    })))
}

pub async fn certification(
    State(validation_runner): State<Arc<ValidationRunner>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, Json<ApiErrorEnvelope>> {
    match validation_runner.run_certification().await {
        Ok(report) => {
            let val = serde_json::to_value(report).unwrap_or(serde_json::json!({}));
            Ok(Json(ApiResponse::success(val)))
        },
        Err(e) => Err(Json(ApiErrorEnvelope::new("INTERNAL_ERROR", &e.to_string()))),
    }
}
