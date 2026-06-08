-- V9: Workflow orchestration and agent registry tables
-- All IDs are UUIDv7 text.

-- ─── Workflows ───────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS workflows (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    current_version INTEGER NOT NULL DEFAULT 1,
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_workflow_status ON workflows(status);

-- ─── Workflow versions (immutable definitions) ───────────────────
CREATE TABLE IF NOT EXISTS workflow_versions (
    id              TEXT PRIMARY KEY,
    workflow_id     TEXT NOT NULL,
    version         INTEGER NOT NULL,
    definition_json TEXT NOT NULL,
    timeout_ms      INTEGER,
    created_at      INTEGER NOT NULL,
    updated_by      TEXT,
    updated_at      INTEGER,
    FOREIGN KEY(workflow_id) REFERENCES workflows(id),
    UNIQUE(workflow_id, version)
);

-- ─── Workflow steps ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS workflow_steps (
    id                  TEXT PRIMARY KEY,
    workflow_version_id TEXT NOT NULL,
    step_order          INTEGER NOT NULL,
    name                TEXT NOT NULL DEFAULT '',
    definition_json     TEXT NOT NULL,
    timeout_ms          INTEGER,
    retry_policy_json   TEXT,
    compensation_json   TEXT,
    created_at          INTEGER NOT NULL,
    updated_by          TEXT,
    updated_at          INTEGER,
    FOREIGN KEY(workflow_version_id) REFERENCES workflow_versions(id)
);

-- ─── Workflow executions ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS workflow_executions (
    id                  TEXT PRIMARY KEY,
    workflow_version_id TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending',
    start_ts            INTEGER,
    end_ts              INTEGER,
    metrics_json        TEXT,
    snapshot_json       TEXT,
    created_at          INTEGER NOT NULL,
    FOREIGN KEY(workflow_version_id) REFERENCES workflow_versions(id)
);
CREATE INDEX IF NOT EXISTS idx_execution_status ON workflow_executions(status);

-- ─── Workflow events (append-only, sequence-numbered) ────────────
CREATE TABLE IF NOT EXISTS workflow_events (
    id              TEXT PRIMARY KEY,
    execution_id    TEXT NOT NULL,
    step_id         TEXT,
    sequence_number INTEGER NOT NULL,
    schema_version  INTEGER NOT NULL DEFAULT 1,
    event_type      TEXT NOT NULL,
    payload_json    TEXT,
    ts              INTEGER NOT NULL,
    FOREIGN KEY(execution_id) REFERENCES workflow_executions(id)
);
CREATE INDEX IF NOT EXISTS idx_wf_event_exec_seq
    ON workflow_events(execution_id, sequence_number);

-- Prevent UPDATE and DELETE on workflow events (append-only)
CREATE TRIGGER IF NOT EXISTS trg_workflow_events_no_update
    BEFORE UPDATE ON workflow_events
    BEGIN SELECT RAISE(ABORT, 'workflow_events is append-only'); END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_events_no_delete
    BEFORE DELETE ON workflow_events
    BEGIN SELECT RAISE(ABORT, 'workflow_events is append-only'); END;

-- ─── Dead letter queue ───────────────────────────────────────────
CREATE TABLE IF NOT EXISTS workflow_dead_letters (
    execution_id        TEXT NOT NULL,
    step_id             TEXT NOT NULL,
    workflow_version_id TEXT NOT NULL DEFAULT '',
    step_name           TEXT NOT NULL DEFAULT '',
    failure_reason      TEXT NOT NULL,
    attempt_count       INTEGER NOT NULL DEFAULT 0,
    last_error          TEXT NOT NULL DEFAULT '',
    last_agent_id       TEXT,
    execution_duration_ms INTEGER NOT NULL DEFAULT 0,
    failed_at           INTEGER NOT NULL,
    created_at          INTEGER NOT NULL,
    PRIMARY KEY (execution_id, step_id)
);

-- ─── Agent registry ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS agent_registry (
    id                TEXT PRIMARY KEY,
    name              TEXT NOT NULL,
    capabilities_json TEXT NOT NULL,
    health_json       TEXT,
    performance_json  TEXT,
    registered_at     INTEGER NOT NULL
);

-- ─── Agent metrics ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS agent_metrics (
    id          TEXT PRIMARY KEY,
    agent_id    TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    metric_value REAL NOT NULL,
    ts          INTEGER NOT NULL,
    FOREIGN KEY(agent_id) REFERENCES agent_registry(id)
);
CREATE INDEX IF NOT EXISTS idx_agent_metrics_agent ON agent_metrics(agent_id, ts);
