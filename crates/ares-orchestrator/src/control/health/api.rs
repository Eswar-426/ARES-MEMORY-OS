use super::worker_health::{calculate_health_score, WorkerHealth};
use crate::control::workers::repository::WorkerRepository;
use ares_core::AresError;
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

fn map_err(e: AresError) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::from_u16(e.ipc_code() as u16)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        e.to_string(),
    )
}

pub struct HealthApiState {
    pub worker_repo: Arc<WorkerRepository>,
}

#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/workers/health",
    responses(
        (status = 200, description = "Worker health scores", body = Vec<WorkerHealth>)
    )
)]
pub async fn get_worker_health(
    State(state): State<Arc<HealthApiState>>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let workers = state.worker_repo.list().map_err(map_err)?;
    let health_scores: Vec<WorkerHealth> = workers
        .into_iter()
        .map(|w| {
            let score = calculate_health_score(&w);
            WorkerHealth {
                worker_id: w.id,
                hostname: w.hostname,
                status: w.status,
                health_score: score,
                available_cpu: w.resources.available_cpu,
                available_memory: w.resources.available_memory,
            }
        })
        .collect();

    Ok(Json(health_scores))
}
