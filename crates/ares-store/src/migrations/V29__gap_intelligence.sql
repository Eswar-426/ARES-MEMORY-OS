-- V29: Gap Intelligence Engine Schema

CREATE TABLE IF NOT EXISTS repository_health_trends (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    snapshot_time INTEGER NOT NULL,
    overall_score REAL NOT NULL,
    component_scores TEXT NOT NULL, -- JSON
    total_gaps INTEGER NOT NULL,
    critical_gaps INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_repo_health_trends ON repository_health_trends(project_id);
