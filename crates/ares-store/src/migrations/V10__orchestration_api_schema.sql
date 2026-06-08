-- V10: Orchestration API, Visualization Cache, Analytics Rollups, and Auditability

-- 1. Workflow Visualizations Cache
CREATE TABLE IF NOT EXISTS workflow_visualizations (
    workflow_version_id TEXT PRIMARY KEY,
    mermaid TEXT,
    graph_json TEXT NOT NULL,
    visualization_truncated BOOLEAN NOT NULL DEFAULT 0,
    generated_at INTEGER NOT NULL,
    FOREIGN KEY(workflow_version_id) REFERENCES workflow_versions(id) ON DELETE CASCADE
);

-- 2. Analytics Rollup Cache
CREATE TABLE IF NOT EXISTS workflow_analytics_cache (
    id INTEGER PRIMARY KEY CHECK (id = 1), -- Ensure single-row table
    total_executions INTEGER NOT NULL DEFAULT 0,
    running_executions INTEGER NOT NULL DEFAULT 0,
    completed_executions INTEGER NOT NULL DEFAULT 0,
    failed_executions INTEGER NOT NULL DEFAULT 0,
    p50_duration_ms REAL NOT NULL DEFAULT 0.0,
    p95_duration_ms REAL NOT NULL DEFAULT 0.0,
    p99_duration_ms REAL NOT NULL DEFAULT 0.0,
    retry_rate REAL NOT NULL DEFAULT 0.0,
    failure_rate REAL NOT NULL DEFAULT 0.0,
    compensation_rate REAL NOT NULL DEFAULT 0.0,
    dead_letter_count INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL
);

-- Initialize the single row
INSERT OR IGNORE INTO workflow_analytics_cache (id, updated_at) VALUES (1, 0);

-- 3. Replay Audit Log
CREATE TABLE IF NOT EXISTS replay_audit_log (
    replay_id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER NOT NULL,
    events_replayed INTEGER NOT NULL,
    checksum_verified BOOLEAN NOT NULL,
    FOREIGN KEY(execution_id) REFERENCES workflow_executions(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_replay_audit_execution ON replay_audit_log(execution_id);
CREATE INDEX IF NOT EXISTS idx_replay_audit_time ON replay_audit_log(started_at);

-- 4. Optimistic Concurrency Control
ALTER TABLE workflow_executions ADD COLUMN version INTEGER NOT NULL DEFAULT 0;

-- 5. Immutable Event Store Triggers
-- Prevent updates to historical events
CREATE TRIGGER IF NOT EXISTS prevent_event_update
BEFORE UPDATE ON workflow_events
BEGIN
    SELECT RAISE(ABORT, 'workflow_events are immutable');
END;

-- Prevent deletes of historical events
CREATE TRIGGER IF NOT EXISTS prevent_event_delete
BEFORE DELETE ON workflow_events
BEGIN
    SELECT RAISE(ABORT, 'workflow_events are immutable');
END;
