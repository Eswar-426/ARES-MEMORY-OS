pub mod context;
pub mod contradictions;
pub mod decisions;
pub mod extractor;
pub mod import;
pub mod memory;
pub mod observability;
pub mod projects;
pub mod snapshot;

pub mod agents;
pub mod knowledge;
pub mod planner;
pub mod reindex;
pub mod scan;
pub mod semantic;
pub mod telemetry;
pub mod workflows;

use ares_core::AresError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

pub fn into_response<T: serde::Serialize>(result: Result<T, AresError>) -> Response {
    match result {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => {
            let status = match e.ipc_code() {
                404 => StatusCode::NOT_FOUND,
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                1001 | 1002 => StatusCode::PRECONDITION_FAILED,
                503 => StatusCode::SERVICE_UNAVAILABLE,
                504 => StatusCode::GATEWAY_TIMEOUT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            let payload = serde_json::json!({
                "error": {
                    "code": e.ipc_code(),
                    "message": e.to_string()
                }
            });
            (status, Json(payload)).into_response()
        }
    }
}
