-- Phase 10A: Requirement Evolution Intelligence
CREATE TABLE requirement_evolution_events (
    id TEXT PRIMARY KEY,
    requirement_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_origin TEXT NOT NULL,
    actor TEXT,
    description TEXT,
    correlation_id TEXT,
    previous_score REAL,
    new_score REAL,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_req_evolution_requirement ON requirement_evolution_events(requirement_id);
CREATE INDEX idx_req_evolution_created ON requirement_evolution_events(created_at);
CREATE INDEX idx_req_evolution_correlation ON requirement_evolution_events(correlation_id);
