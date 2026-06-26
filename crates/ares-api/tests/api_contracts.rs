use ares_api::models::{ApiErrorEnvelope, ApiResponse};

#[test]
fn test_api_response_envelope() {
    let payload = serde_json::json!({"entity": "test-entity"});
    let response = ApiResponse::success(payload);

    assert_eq!(response.status, "success");
    assert!(response.request_id.starts_with("REQ-"));
    assert_eq!(response.data["entity"], "test-entity");
}

#[test]
fn test_api_error_envelope() {
    let error = ApiErrorEnvelope::new("NOT_FOUND", "Entity not found");

    assert_eq!(error.status, "error");
    assert!(error.request_id.starts_with("REQ-"));
    assert_eq!(error.error.code, "NOT_FOUND");
    assert_eq!(error.error.message, "Entity not found");
}
