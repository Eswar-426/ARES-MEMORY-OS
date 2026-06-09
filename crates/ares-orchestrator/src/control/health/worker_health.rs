use crate::control::workers::models::{Worker, WorkerStatus};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct WorkerHealth {
    pub worker_id: String,
    pub hostname: String,
    pub status: WorkerStatus,
    pub health_score: u8,
    pub available_cpu: f32,
    pub available_memory: u64,
}

pub fn calculate_health_score(worker: &Worker) -> u8 {
    if worker.status == WorkerStatus::Dead || worker.status == WorkerStatus::Offline {
        return 0;
    }

    let mut score: i32 = 100;

    // Deduct points for high load
    if worker.resources.cpu > 0.0 {
        let cpu_usage_pct = 1.0 - (worker.resources.available_cpu / worker.resources.cpu);
        if cpu_usage_pct > 0.9 {
            score -= 20;
        } else if cpu_usage_pct > 0.7 {
            score -= 10;
        }
    }

    if worker.resources.memory > 0 {
        let mem_usage_pct =
            1.0 - (worker.resources.available_memory as f64 / worker.resources.memory as f64);
        if mem_usage_pct > 0.9 {
            score -= 20;
        } else if mem_usage_pct > 0.7 {
            score -= 10;
        }
    }

    // Deduct points for stale heartbeat (even if not fully timed out)
    let now = Utc::now();
    if let Ok(last_hb) = DateTime::parse_from_rfc3339(&worker.last_heartbeat) {
        let duration = now
            .signed_duration_since(last_hb.with_timezone(&Utc))
            .num_seconds();
        if duration > 15 {
            score -= 30; // Missing multiple heartbeats is bad
        } else if duration > 5 {
            score -= 5;
        }
    }

    score.clamp(0, 100) as u8
}
