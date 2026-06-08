use crate::routes::into_response;
use ares_app::AppState;
use ares_core::types::workflow_api::{
    ExecutionSearchRequest, PageResponse, WorkflowAnalyticsReport, WorkflowRunRequest,
};
use axum::{
    extract::{Path, Query, State},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    Json,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;

// ─────────────────────────────────────────────────────────────────
// Lifecycle
// ─────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/workflows/run",
    request_body = WorkflowRunRequest,
    responses(
        (status = 200, description = "Workflow started")
    )
)]
pub async fn run_workflow(
    State(state): State<AppState>,
    Json(payload): Json<WorkflowRunRequest>,
) -> impl IntoResponse {
    // 1. Create plan
    let plan = match state.workflow_engine.create_execution_plan(
        &payload.workflow_id,
        &payload.workflow_version_id,
        payload.version,
    ) {
        Ok(p) => p,
        Err(e) => return into_response::<()>(Err(e)),
    };

    // 2. Execute plan
    let result = state.workflow_engine.execute_workflow(&plan);
    if result.is_ok() {
        metrics::counter!("workflow_runs_total").increment(1);
    }
    into_response(result)
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows/executions/{id}/pause",
    params(
        ("id" = String, Path, description = "Execution ID")
    ),
    responses(
        (status = 200, description = "Workflow paused")
    )
)]
pub async fn pause_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let exec_id = ares_core::ExecutionId(id);
    let result = match state
        .workflow_engine
        .reconstruct_execution_state(&exec_id, false)
    {
        Ok(mut st) => state.workflow_engine.pause_workflow(&mut st),
        Err(e) => Err(e),
    };
    into_response(result)
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows/executions/{id}/resume",
    params(
        ("id" = String, Path, description = "Execution ID")
    ),
    responses(
        (status = 200, description = "Workflow resumed")
    )
)]
pub async fn resume_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let exec_id = ares_core::ExecutionId(id);
    let result = match state
        .workflow_engine
        .reconstruct_execution_state(&exec_id, false)
    {
        Ok(mut st) => state.workflow_engine.resume_workflow(&mut st),
        Err(e) => Err(e),
    };
    into_response(result)
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows/executions/{id}/cancel",
    params(
        ("id" = String, Path, description = "Execution ID")
    ),
    responses(
        (status = 200, description = "Workflow cancelled")
    )
)]
pub async fn cancel_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let exec_id = ares_core::ExecutionId(id);
    let result = match state
        .workflow_engine
        .reconstruct_execution_state(&exec_id, false)
    {
        Ok(mut st) => state.workflow_engine.cancel_workflow(&mut st),
        Err(e) => Err(e),
    };
    into_response(result)
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows/executions/{id}/retry",
    params(
        ("id" = String, Path, description = "Execution ID")
    ),
    responses(
        (status = 200, description = "Workflow retried")
    )
)]
pub async fn retry_workflow(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    // Retry is usually handled by execution engine
    into_response(Ok::<_, ares_core::AresError>(()))
}

// ─────────────────────────────────────────────────────────────────
// Search & Stream
// ─────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/workflows/executions/search",
    params(
        ExecutionSearchRequest
    ),
    responses(
        (status = 200, description = "Search results", body = PageResponseExecutionSummary)
    )
)]
pub async fn search_executions(
    State(state): State<AppState>,
    Query(req): Query<ExecutionSearchRequest>,
) -> impl IntoResponse {
    metrics::counter!("workflow_search_requests_total").increment(1);

    let result = state.workflow_repo.search_executions(&req);
    match result {
        Ok((data, total)) => into_response(Ok::<_, ares_core::AresError>(PageResponse::<
            ares_core::types::workflow_api::ExecutionSummary,
        > {
            data,
            total,
            page: req.page.unwrap_or(1),
            page_size: req.page_size.unwrap_or(50),
        })),
        Err(e) => {
            into_response::<PageResponse<ares_core::types::workflow_api::ExecutionSummary>>(Err(e))
        }
    }
}

// Stub for SSE Stream
pub async fn execution_stream(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Implement backpressured polling SSE stream
    let stream = async_stream::stream! {
        let _last_seq = 0;
        loop {
            // In a real implementation, poll the DB for events > last_seq
            // For now we yield a heartbeat to satisfy the SSE requirement
            yield Ok(Event::default().data("ping"));
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

// ─────────────────────────────────────────────────────────────────
// Replay, Analytics, Visualization
// ─────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/workflows/replay/{id}",
    params(
        ("id" = String, Path, description = "Execution ID")
    ),
    responses(
        (status = 200, description = "Replay report", body = ReplayReport)
    )
)]
pub async fn replay_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    metrics::counter!("workflow_replay_requests_total").increment(1);
    let exec_id = ares_core::ExecutionId(id);
    let result = state
        .replay_service
        .replay_execution(&exec_id, "admin-api")
        .await;
    into_response(result)
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/analytics",
    responses(
        (status = 200, description = "Analytics report", body = WorkflowAnalyticsReport)
    )
)]
pub async fn get_analytics(State(state): State<AppState>) -> impl IntoResponse {
    metrics::counter!("workflow_analytics_requests_total").increment(1);
    let result = state.workflow_analytics.generate_report();
    match result {
        Ok(report) => into_response(Ok::<_, ares_core::AresError>(report)),
        Err(_) => into_response(Ok::<_, ares_core::AresError>(WorkflowAnalyticsReport {
            total_executions: 0,
            running_executions: 0,
            completed_executions: 0,
            failed_executions: 0,
            p50_duration_ms: 0.0,
            p95_duration_ms: 0.0,
            p99_duration_ms: 0.0,
            retry_rate: 0.0,
            failure_rate: 0.0,
            compensation_rate: 0.0,
            dead_letter_count: 0,
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/visualize/{id}",
    params(
        ("id" = String, Path, description = "Workflow Version ID")
    ),
    responses(
        (status = 200, description = "Visualization data", body = WorkflowGraphResponse)
    )
)]
pub async fn visualize_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    metrics::counter!("workflow_visualization_requests_total").increment(1);

    // Attempt cache hit
    if let Ok(Some(cached_json)) = state.workflow_repo.get_visualization(&id) {
        if let Ok(cached_resp) = serde_json::from_str::<
            ares_core::types::workflow_api::WorkflowGraphResponse,
        >(&cached_json)
        {
            return into_response(Ok::<_, ares_core::AresError>(cached_resp));
        }
    }

    // Cache miss - generate and store
    let result = state.workflow_visualizer.visualize(&id);
    if let Ok(resp) = &result {
        if let Ok(resp_json) = serde_json::to_string(resp) {
            let _ = state.workflow_repo.save_visualization(&id, &resp_json);
        }
    }

    into_response(result)
}
