use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ApiResponse<T> {
    pub status: String,
    pub request_id: String,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            request_id: format!("REQ-{}", Uuid::new_v4()),
            data,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ApiErrorEnvelope {
    pub status: String,
    pub request_id: String,
    pub error: ApiErrorDetail,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
}

impl ApiErrorEnvelope {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            status: "error".to_string(),
            request_id: format!("REQ-{}", Uuid::new_v4()),
            error: ApiErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct EvolutionResult {
    pub entity: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct MemoryContextPackage {
    pub entity: String,
    pub context: String,
}
