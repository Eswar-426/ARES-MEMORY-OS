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

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::Lazy;

static RECORDER: Lazy<PrometheusHandle> = Lazy::new(|| {
    let builder = PrometheusBuilder::new();
    builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
});

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics")
    )
)]
pub async fn metrics() -> String {
    RECORDER.render()
}
