-- Week 16: Experience Learning & Self-Improvement Tables

CREATE TABLE IF NOT EXISTS mission_outcomes (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    strategy_used TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    score REAL NOT NULL,
    cost REAL NOT NULL,
    duration_secs REAL NOT NULL,
    completed_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mission_id) REFERENCES missions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_mission_outcomes_mission
    ON mission_outcomes(mission_id);
CREATE INDEX IF NOT EXISTS idx_mission_outcomes_strategy
    ON mission_outcomes(strategy_used);

CREATE TABLE IF NOT EXISTS strategy_history (
    id TEXT PRIMARY KEY,
    strategy TEXT NOT NULL UNIQUE,
    ema_success_rate REAL NOT NULL DEFAULT 0.0,
    ema_cost REAL NOT NULL DEFAULT 0.0,
    ema_duration REAL NOT NULL DEFAULT 0.0,
    sample_count INTEGER NOT NULL DEFAULT 0,
    last_updated DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_strategy_history_strategy
    ON strategy_history(strategy);

CREATE TABLE IF NOT EXISTS learning_profiles (
    id TEXT PRIMARY KEY,
    profile_json TEXT NOT NULL,
    total_missions INTEGER NOT NULL DEFAULT 0,
    overall_ema_score REAL NOT NULL DEFAULT 0.0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
