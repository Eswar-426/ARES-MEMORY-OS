use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthStatus)
    )
)]
pub async fn health() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics")
    )
)]
pub async fn metrics() -> String {
    // Return mock metrics for now, or real metrics if registry is wired
    "# HELP ares_requests_total Total number of HTTP requests\n# TYPE ares_requests_total counter\n".into()
}
