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

pub fn init_metrics() {
    Lazy::force(&RECORDER);
    metrics::counter!("ares_app_starts_total").increment(1);

    // Eagerly initialize metrics to 0 so they appear in Prometheus
    metrics::counter!("workflow_runs_total").absolute(0);
    metrics::counter!("workflow_search_requests_total").absolute(0);
    metrics::counter!("workflow_replay_requests_total").absolute(0);
    metrics::counter!("workflow_analytics_requests_total").absolute(0);
    metrics::counter!("workflow_visualization_requests_total").absolute(0);
    metrics::counter!("agent_registrations_total").absolute(0);
    metrics::counter!("agent_list_requests_total").absolute(0);
    metrics::counter!("agent_heartbeat_total").absolute(0);

    // Orchestrator metrics
    metrics::counter!("orchestrator_worker_registrations_total").absolute(0);
    metrics::counter!("orchestrator_heartbeats_total").absolute(0);
    metrics::counter!("orchestrator_job_enqueues_total").absolute(0);
    metrics::counter!("orchestrator_dlq_events_total").absolute(0);
    metrics::gauge!("orchestrator_active_workers").set(0.0);
    metrics::gauge!("orchestrator_queue_depth").set(0.0);
}

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
