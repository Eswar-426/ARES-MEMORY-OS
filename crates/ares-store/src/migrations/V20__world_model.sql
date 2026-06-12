-- V20: World Model, Predictive Planning & Simulation Engine
-- Week 18 — Tables for the predictive autonomous operating system

-- 1. World State snapshots
CREATE TABLE IF NOT EXISTS world_states (
    id              TEXT PRIMARY KEY,
    goals_json      TEXT NOT NULL DEFAULT '[]',
    resources_json  TEXT NOT NULL DEFAULT '[]',
    agents_json     TEXT NOT NULL DEFAULT '[]',
    constraints_json TEXT NOT NULL DEFAULT '[]',
    snapshot_at     INTEGER NOT NULL,
    created_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_world_states_snapshot_at ON world_states(snapshot_at);

-- 2. Generated scenarios (possible futures)
CREATE TABLE IF NOT EXISTS scenarios (
    id                      TEXT PRIMARY KEY,
    goal_id                 TEXT NOT NULL,
    scenario_type           TEXT NOT NULL,
    description             TEXT NOT NULL DEFAULT '',
    estimated_cost          REAL NOT NULL DEFAULT 0.0,
    estimated_duration_secs REAL NOT NULL DEFAULT 0.0,
    estimated_quality       REAL NOT NULL DEFAULT 0.0,
    agent_assignments_json  TEXT NOT NULL DEFAULT '[]',
    steps_json              TEXT NOT NULL DEFAULT '[]',
    world_state_id          TEXT,
    created_at              INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (world_state_id) REFERENCES world_states(id)
);

CREATE INDEX IF NOT EXISTS idx_scenarios_goal_id ON scenarios(goal_id);
CREATE INDEX IF NOT EXISTS idx_scenarios_type ON scenarios(scenario_type);

-- 3. Simulation results
CREATE TABLE IF NOT EXISTS simulations (
    id                      TEXT PRIMARY KEY,
    scenario_id             TEXT NOT NULL,
    task_duration_secs      REAL NOT NULL DEFAULT 0.0,
    total_cost              REAL NOT NULL DEFAULT 0.0,
    success_probability     REAL NOT NULL DEFAULT 0.0,
    agent_utilization       REAL NOT NULL DEFAULT 0.0,
    memory_usage_estimate   REAL NOT NULL DEFAULT 0.0,
    risk_score              REAL NOT NULL DEFAULT 0.0,
    config_json             TEXT NOT NULL DEFAULT '{}',
    simulated_at            INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (scenario_id) REFERENCES scenarios(id)
);

CREATE INDEX IF NOT EXISTS idx_simulations_scenario_id ON simulations(scenario_id);

-- 4. Risk reports
CREATE TABLE IF NOT EXISTS risk_reports (
    id                          TEXT PRIMARY KEY,
    scenario_id                 TEXT NOT NULL,
    overall_risk                TEXT NOT NULL DEFAULT 'Low',
    failure_probability         REAL NOT NULL DEFAULT 0.0,
    budget_overrun_probability  REAL NOT NULL DEFAULT 0.0,
    resource_exhaustion_risk    REAL NOT NULL DEFAULT 0.0,
    dependency_risk             REAL NOT NULL DEFAULT 0.0,
    execution_risk              REAL NOT NULL DEFAULT 0.0,
    risk_factors_json           TEXT NOT NULL DEFAULT '[]',
    mitigations_json            TEXT NOT NULL DEFAULT '[]',
    analyzed_at                 INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (scenario_id) REFERENCES scenarios(id)
);

CREATE INDEX IF NOT EXISTS idx_risk_reports_scenario_id ON risk_reports(scenario_id);

-- 5. Outcome predictions
CREATE TABLE IF NOT EXISTS predictions (
    id                      TEXT PRIMARY KEY,
    goal_id                 TEXT NOT NULL,
    scenario_id             TEXT,
    success_probability     REAL NOT NULL DEFAULT 0.0,
    estimated_cost          REAL NOT NULL DEFAULT 0.0,
    estimated_duration_secs REAL NOT NULL DEFAULT 0.0,
    confidence              REAL NOT NULL DEFAULT 0.0,
    confidence_reasons_json TEXT NOT NULL DEFAULT '[]',
    similar_mission_count   INTEGER NOT NULL DEFAULT 0,
    prediction_method       TEXT NOT NULL DEFAULT 'deterministic',
    predicted_at            INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (scenario_id) REFERENCES scenarios(id)
);

CREATE INDEX IF NOT EXISTS idx_predictions_goal_id ON predictions(goal_id);

-- 6. Forecast history (predicted vs actual for learning)
CREATE TABLE IF NOT EXISTS forecast_history (
    id                      TEXT PRIMARY KEY,
    prediction_id           TEXT NOT NULL,
    predicted_cost          REAL NOT NULL DEFAULT 0.0,
    actual_cost             REAL NOT NULL DEFAULT 0.0,
    predicted_duration_secs REAL NOT NULL DEFAULT 0.0,
    actual_duration_secs    REAL NOT NULL DEFAULT 0.0,
    predicted_success       REAL NOT NULL DEFAULT 0.0,
    actual_success          INTEGER NOT NULL DEFAULT 0,
    deviation_score         REAL NOT NULL DEFAULT 0.0,
    recorded_at             INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (prediction_id) REFERENCES predictions(id)
);

CREATE INDEX IF NOT EXISTS idx_forecast_history_prediction_id ON forecast_history(prediction_id);
CREATE INDEX IF NOT EXISTS idx_forecast_history_recorded_at ON forecast_history(recorded_at);

-- 7. Strategy rankings
CREATE TABLE IF NOT EXISTS strategy_rankings (
    id              TEXT PRIMARY KEY,
    goal_id         TEXT NOT NULL,
    scenario_id     TEXT NOT NULL,
    rank            INTEGER NOT NULL DEFAULT 0,
    composite_score REAL NOT NULL DEFAULT 0.0,
    speed_score     REAL NOT NULL DEFAULT 0.0,
    quality_score   REAL NOT NULL DEFAULT 0.0,
    cost_score      REAL NOT NULL DEFAULT 0.0,
    risk_score      REAL NOT NULL DEFAULT 0.0,
    success_score   REAL NOT NULL DEFAULT 0.0,
    explanation     TEXT NOT NULL DEFAULT '',
    ranked_at       INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (scenario_id) REFERENCES scenarios(id)
);

CREATE INDEX IF NOT EXISTS idx_strategy_rankings_goal_id ON strategy_rankings(goal_id);
CREATE INDEX IF NOT EXISTS idx_strategy_rankings_rank ON strategy_rankings(goal_id, rank);
