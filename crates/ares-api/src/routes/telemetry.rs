use ares_app::AppState;
use ares_store::repositories::telemetry::{TelemetryReport, TelemetryRepository};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct TelemetryResponse {
    pub id: String,
    pub timestamp: String,
    pub source: String,
    pub continuity_score: f64,
    pub provider_health: Value,
    pub fallback_events: Value,
    pub dynamic_chains: Value,
}

impl From<TelemetryReport> for TelemetryResponse {
    fn from(report: TelemetryReport) -> Self {
        Self {
            id: report.id,
            timestamp: report.timestamp,
            source: report.source,
            continuity_score: report.continuity_score,
            provider_health: report.provider_health,
            fallback_events: report.fallback_events,
            dynamic_chains: report.dynamic_chains,
        }
    }
}

pub async fn get_latest_telemetry(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let path_env = std::env::var("ARES_TELEMETRY_DB_PATH").ok();
    tracing::error!("ARES_TELEMETRY_DB_PATH is {:?}", path_env);

    let store = if let Some(path) = &path_env {
        tracing::error!("Opening DB from path: {}", path);
        ares_store::db::Store::open(std::path::Path::new(path)).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open benchmark DB: {}", e),
            )
        })?
    } else {
        tracing::error!("Using state.store");
        state.store.clone()
    };

    let repo = TelemetryRepository::new(&store);
    let mut report_opt = repo.get_latest_report().ok().flatten();
    tracing::error!(
        "Report from first attempt: is_some={}",
        report_opt.is_some()
    );

    if report_opt.is_none() && path_env.is_none() {
        tracing::error!("Falling back to scratch/benchmark.db");
        match ares_store::db::Store::open(std::path::Path::new("scratch/benchmark.db")) {
            Ok(fallback_store) => {
                let fallback_repo = TelemetryRepository::new(&fallback_store);
                report_opt = fallback_repo.get_latest_report().ok().flatten();
                tracing::error!(
                    "Report from fallback attempt: is_some={}",
                    report_opt.is_some()
                );
            }
            Err(e) => {
                tracing::error!("Failed to open fallback DB: {}", e);
            }
        }
    }

    match report_opt {
        Some(report) => {
            tracing::error!("Returning valid report!");
            let response: TelemetryResponse = TelemetryResponse {
                id: report.id,
                timestamp: report.timestamp,
                source: report.source,
                continuity_score: report.continuity_score,
                provider_health: report.provider_health,
                fallback_events: report.fallback_events,
                dynamic_chains: report.dynamic_chains,
            };
            Ok(Json(response))
        }
        None => {
            tracing::error!("Returning 404 because report_opt is None");
            Err((
                StatusCode::NOT_FOUND,
                "No telemetry reports found".to_string(),
            ))
        }
    }
}
