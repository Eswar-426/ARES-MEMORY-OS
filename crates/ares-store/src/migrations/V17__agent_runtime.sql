-- Week 15: Agent Runtime & Autonomous Execution Engine Tables

CREATE TABLE IF NOT EXISTS missions (
    id TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    dag_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS mission_events (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mission_id) REFERENCES missions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS mission_checkpoints (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    state_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mission_id) REFERENCES missions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS agent_runtime (
    id TEXT PRIMARY KEY,
    role TEXT NOT NULL,
    config_json TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS agent_assignments (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mission_id) REFERENCES missions(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_id) REFERENCES agent_runtime(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS execution_history (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    result_json TEXT NOT NULL,
    error_msg TEXT,
    started_at DATETIME,
    completed_at DATETIME,
    FOREIGN KEY (mission_id) REFERENCES missions(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_id) REFERENCES agent_runtime(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS reflection_reports (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    task_id TEXT,
    agent_id TEXT,
    report_json TEXT NOT NULL,
    quality_score INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mission_id) REFERENCES missions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS agent_memory (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    memory_key TEXT NOT NULL,
    memory_value TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (agent_id) REFERENCES agent_runtime(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS mission_memory (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    memory_key TEXT NOT NULL,
    memory_value TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mission_id) REFERENCES missions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS execution_metrics (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    task_id TEXT,
    agent_id TEXT,
    duration_ms INTEGER NOT NULL,
    success BOOLEAN NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS resource_usage (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    cpu_cores INTEGER NOT NULL,
    memory_mb INTEGER NOT NULL,
    tokens INTEGER NOT NULL,
    api_calls INTEGER NOT NULL,
    recorded_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mission_id) REFERENCES missions(id) ON DELETE CASCADE
);
