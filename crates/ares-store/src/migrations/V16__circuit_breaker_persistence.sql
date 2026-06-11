-- V16: Circuit Breaker Persistence

CREATE TABLE IF NOT EXISTS circuit_breaker_states (
    provider_id TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    failure_count INTEGER NOT NULL,
    opened_at DATETIME
);
