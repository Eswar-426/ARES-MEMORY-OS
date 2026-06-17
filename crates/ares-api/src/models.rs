use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[aliases(
    ApiResponseHealthStatus = ApiResponse<HealthStatus>,
    ApiResponseValue = ApiResponse<serde_json::Value>,
    ApiResponseEvolutionResult = ApiResponse<EvolutionResult>,
    ApiResponseMemoryContextPackage = ApiResponse<MemoryContextPackage>,
    ApiResponseCertification = ApiResponse<serde_json::Value>
)]
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

// ─────────────────────────────────────────────────────────────────
// Concrete DTOs to replace ares_core::types::pagination::Page<T>
// ─────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct GraphNodePageResponse {
    #[schema(value_type = Vec<Object>)]
    pub items: Vec<ares_core::GraphNode>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct TimelinePageResponse {
    #[schema(value_type = Vec<Object>)]
    pub items: Vec<ares_core::AresEvent>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DecisionPageResponse {
    #[schema(value_type = Vec<Object>)]
    pub items: Vec<ares_core::Decision>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}
