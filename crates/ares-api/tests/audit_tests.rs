use ares_agent::config::AgentConfig;
use ares_api::create_router;
use ares_app::AppState;
use ares_core::{
    types::workflow_api::WorkflowRunRequest, AgentHealth, AgentId, AgentInfo, WorkflowId,
};
use axum::{
    body::Body,
    http::{Method, Request},
};
use rusqlite::Connection;
use tempfile::TempDir;
use tower::ServiceExt;

async fn setup_app() -> (axum::Router, Connection, TempDir) {
    std::env::set_var("ARES_AUTH_DISABLED", "true");
    let temp_dir = tempfile::tempdir().unwrap();
    let config = AgentConfig::load(temp_dir.path().to_str().unwrap()).unwrap();

    // ensure .ares dir exists
    std::fs::create_dir_all(temp_dir.path().join(".ares")).unwrap();

    let state = AppState::new(config).await.unwrap();
    let conn = Connection::open(temp_dir.path().join(".ares").join("ares.db")).unwrap();

    // Init metrics so it doesn't crash on multiple tests
    ares_api::routes::observability::init_metrics();

    let router = create_router(state);
    (router, conn, temp_dir)
}

fn count_rows(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
        row.get(0)
    })
    .unwrap_or(0)
}

#[tokio::test]
async fn phase4_audit_test() {
    let (app, conn, _dir) = setup_app().await;

    println!("\n=== PHASE 4 API AUDIT ===");

    // 1. Register Agent
    println!("\n[1] POST /api/v1/agents/register");
    let rows_before = count_rows(&conn, "agent_registry");
    let agent_id = AgentId::new();
    let info = AgentInfo {
        id: agent_id.clone(),
        name: "TestAgent".into(),
        capabilities: vec![],
        health: AgentHealth {
            is_available: true,
            health_score: 1.0,
            last_check: 0,
            consecutive_failures: 0,
        },
        performance: Default::default(),
        registered_at: 0,
    };
    let body = Body::from(serde_json::to_vec(&info).unwrap());
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/agents/register")
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let rows_after = count_rows(&conn, "agent_registry");
    println!("Response: {}", resp.status());
    println!("SQLite rows before: {}", rows_before);
    println!("SQLite rows after: {}", rows_after);
    assert_eq!(rows_after, rows_before + 1);

    // 2. List Agents
    println!("\n[2] GET /api/v1/agents");
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/agents")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    println!("Response: {}", resp.status());
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    println!("Payload: {:?}", std::str::from_utf8(&body_bytes).unwrap());

    // 3. Heartbeat
    println!("\n[3] POST /api/v1/agents/{{id}}/heartbeat");
    let health = AgentHealth {
        is_available: true,
        health_score: 0.99,
        last_check: 12345,
        consecutive_failures: 0,
    };
    let body = Body::from(serde_json::to_vec(&health).unwrap());
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/v1/agents/{}/heartbeat", agent_id.as_str()))
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    println!("Response: {}", resp.status());

    // 4. Run Workflow
    println!("\n[4] POST /api/v1/workflows/run");
    let wf_id = WorkflowId::new();
    // Pre-insert workflow into db so plan can execute
    let now = ares_core::types::event::now_micros();
    conn.execute("INSERT INTO workflows (id, name, description, current_version, status, created_at, updated_at) VALUES (?1, 'test', 'desc', 1, 'active', ?2, ?2)", rusqlite::params![&wf_id.as_str(), now as i64]).unwrap();
    let def_json = format!(
        r#"{{"workflow_id": "{}", "version": 1, "name": "test", "description": "desc", "steps": []}}"#,
        wf_id.as_str()
    );
    conn.execute("INSERT INTO workflow_versions (id, workflow_id, version, definition_json, created_at) VALUES (?1, ?2, 1, ?3, ?4)", rusqlite::params!["test_v1", wf_id.as_str(), &def_json, now as i64]).unwrap();

    let rows_before = count_rows(&conn, "workflow_executions");
    let req_payload = WorkflowRunRequest {
        workflow_id: wf_id.clone(),
        workflow_version_id: "test_v1".into(),
        version: 1,
    };
    let body = Body::from(serde_json::to_vec(&req_payload).unwrap());
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/workflows/run")
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let rows_after = count_rows(&conn, "workflow_executions");
    println!("Response: {}", resp.status());
    println!("SQLite rows before: {}", rows_before);
    println!("SQLite rows after: {}", rows_after);
    assert_eq!(rows_after, rows_before + 1);

    // 5. Search Executions
    println!("\n[5] GET /api/v1/workflows/executions/search");
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/workflows/executions/search")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    println!("Response: {}", resp.status());
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    println!("Payload: {:?}", std::str::from_utf8(&body_bytes).unwrap());

    // 6. Analytics
    println!("\n[6] GET /api/v1/workflows/analytics");
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/workflows/analytics")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    println!("Response: {}", resp.status());

    // 7. Visualize
    println!("\n[7] GET /api/v1/workflows/visualize/test_v1");
    let rows_before = count_rows(&conn, "workflow_visualizations");
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/workflows/visualize/test_v1")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let rows_after = count_rows(&conn, "workflow_visualizations");
    println!("Response: {}", resp.status());
    println!("SQLite rows before: {}", rows_before);
    println!("SQLite rows after: {}", rows_after);
    assert_eq!(rows_after, rows_before + 1);

    // 8. Metrics
    println!("\n[8] GET /metrics");
    let req = Request::builder()
        .method(Method::GET)
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    println!("Response: {}", resp.status());
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics = std::str::from_utf8(&body_bytes).unwrap();
    println!(
        "Metrics contains 'workflow_runs_total': {}",
        metrics.contains("workflow_runs_total")
    );
    println!(
        "Metrics contains 'agent_registrations_total': {}",
        metrics.contains("agent_registrations_total")
    );

    println!("\n=== AUDIT COMPLETE ===\n");
}
