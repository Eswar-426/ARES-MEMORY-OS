-- V15: Multi-Model Intelligence Layer

CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    version TEXT NOT NULL,
    max_context_window INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS model_capabilities (
    model_id TEXT NOT NULL,
    capability TEXT NOT NULL,
    FOREIGN KEY(model_id) REFERENCES models(id)
);

CREATE TABLE IF NOT EXISTS providers (
    id TEXT PRIMARY KEY,
    base_url TEXT NOT NULL,
    api_key_secret_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_health (
    provider_id TEXT NOT NULL,
    status TEXT NOT NULL,
    last_checked_at DATETIME NOT NULL,
    FOREIGN KEY(provider_id) REFERENCES providers(id)
);

CREATE TABLE IF NOT EXISTS routing_history (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    selected_model_id TEXT NOT NULL,
    fallback_model_id TEXT,
    reason TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS model_benchmarks (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    score REAL NOT NULL,
    latency_ms INTEGER NOT NULL,
    FOREIGN KEY(model_id) REFERENCES models(id)
);

CREATE TABLE IF NOT EXISTS model_costs (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    total_cost REAL NOT NULL,
    FOREIGN KEY(model_id) REFERENCES models(id)
);

CREATE TABLE IF NOT EXISTS model_feedback (
    execution_id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    quality_score REAL,
    hallucination_detected BOOLEAN NOT NULL,
    FOREIGN KEY(model_id) REFERENCES models(id)
);

CREATE TABLE IF NOT EXISTS execution_traces (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    latency_ms INTEGER NOT NULL,
    success BOOLEAN NOT NULL,
    FOREIGN KEY(model_id) REFERENCES models(id)
);

CREATE TABLE IF NOT EXISTS fallback_events (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    original_model_id TEXT NOT NULL,
    fallback_model_id TEXT NOT NULL,
    reason TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS model_profiles (
    model_id TEXT PRIMARY KEY,
    success_rate REAL NOT NULL,
    average_latency_ms INTEGER NOT NULL,
    total_executions INTEGER NOT NULL,
    FOREIGN KEY(model_id) REFERENCES models(id)
);

CREATE TABLE IF NOT EXISTS prompt_classifications (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    required_capabilities TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS model_explanations (
    id TEXT PRIMARY KEY,
    decision_type TEXT NOT NULL,
    model_id TEXT NOT NULL,
    explanation TEXT NOT NULL,
    FOREIGN KEY(model_id) REFERENCES models(id)
);

CREATE TABLE IF NOT EXISTS collaboration_runs (
    id TEXT PRIMARY KEY,
    strategy TEXT NOT NULL,
    task_id TEXT NOT NULL,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS context_windows (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    original_tokens INTEGER NOT NULL,
    compressed_tokens INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS intelligence_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS model_policies (
    id TEXT PRIMARY KEY,
    policy_type TEXT NOT NULL,
    constraint_value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ensemble_results (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    consensus_score REAL NOT NULL,
    conflict_detected BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS ab_experiments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    winner_model_id TEXT,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS model_discovery (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    discovered_at DATETIME NOT NULL
);
