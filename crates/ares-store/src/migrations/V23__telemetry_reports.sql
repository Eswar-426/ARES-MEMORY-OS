CREATE TABLE IF NOT EXISTS telemetry_reports (
    id TEXT PRIMARY KEY,
    timestamp DATETIME NOT NULL,
    source TEXT NOT NULL,
    continuity_score REAL NOT NULL,
    provider_health JSON NOT NULL,
    fallback_events JSON NOT NULL,
    dynamic_chains JSON NOT NULL
);
