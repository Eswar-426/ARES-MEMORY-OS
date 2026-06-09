CREATE TABLE workers (
    id TEXT PRIMARY KEY,
    hostname TEXT NOT NULL,
    capabilities TEXT NOT NULL,
    labels TEXT NOT NULL,
    status TEXT NOT NULL,
    resources TEXT NOT NULL,
    registered_at TEXT NOT NULL,
    last_heartbeat TEXT NOT NULL
);

CREATE TABLE workflow_queue (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    priority INTEGER NOT NULL,
    status TEXT NOT NULL,
    assigned_worker TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    execution_key TEXT NOT NULL UNIQUE,
    execution_checksum TEXT NOT NULL
);

CREATE INDEX idx_workflow_queue_status ON workflow_queue(status);
CREATE INDEX idx_workflow_queue_workflow_id ON workflow_queue(workflow_id);
CREATE INDEX idx_workflow_queue_assigned_worker ON workflow_queue(assigned_worker);
CREATE INDEX idx_workflow_queue_priority ON workflow_queue(priority);
CREATE INDEX idx_workflow_queue_created_at ON workflow_queue(created_at);

CREATE TABLE job_leases (
    id TEXT PRIMARY KEY,
    worker_id TEXT NOT NULL,
    queue_id TEXT NOT NULL UNIQUE,
    workflow_id TEXT NOT NULL,
    execution_id TEXT NOT NULL,
    acquired_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);

CREATE INDEX idx_job_leases_expires_at ON job_leases(expires_at);

CREATE TABLE distributed_executions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    completed_at TEXT
);

CREATE TABLE distributed_execution_attempts (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL,
    worker_id TEXT NOT NULL,
    lease_id TEXT NOT NULL,
    attempt_number INTEGER NOT NULL,
    assigned_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    execution_duration_ms INTEGER,
    execution_node TEXT NOT NULL,
    status TEXT NOT NULL,
    error_message TEXT
);

CREATE TABLE workflow_execution_steps (
    id TEXT PRIMARY KEY,
    attempt_id TEXT NOT NULL,
    step_name TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT
);

CREATE TABLE dead_letter_queue (
    id TEXT PRIMARY KEY,
    original_queue_id TEXT NOT NULL,
    workflow_id TEXT NOT NULL,
    execution_key TEXT NOT NULL,
    failure_reason TEXT NOT NULL,
    failed_at TEXT NOT NULL,
    attempt_count INTEGER NOT NULL
);

CREATE TABLE outbox_events (
    id TEXT PRIMARY KEY,
    topic TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL,
    published_at TEXT,
    status TEXT NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_outbox_events_status ON outbox_events(status);
