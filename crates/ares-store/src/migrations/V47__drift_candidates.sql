-- V47__drift_candidates.sql
-- Create drift_candidates table to store drift candidates and their evidence.

CREATE TABLE IF NOT EXISTS drift_candidates (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    target_node_id TEXT NOT NULL REFERENCES graph_nodes(id),
    drift_type TEXT NOT NULL,
    confidence REAL NOT NULL,
    evidence_ids TEXT NOT NULL, -- JSON array
    rationale TEXT NOT NULL,
    detected_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_drift_candidates_project ON drift_candidates(project_id);
CREATE INDEX IF NOT EXISTS idx_drift_candidates_target ON drift_candidates(target_node_id);
