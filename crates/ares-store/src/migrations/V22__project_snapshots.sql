-- V22: Project Snapshots & Chat Imports for Week 20 Productization

CREATE TABLE IF NOT EXISTS project_snapshots (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL,
    snapshot_json TEXT NOT NULL,
    version     INTEGER NOT NULL DEFAULT 1,
    created_at  INTEGER NOT NULL,

    FOREIGN KEY (project_id) REFERENCES projects(id)
);

CREATE INDEX IF NOT EXISTS idx_project_snapshots_project
    ON project_snapshots(project_id, created_at DESC);

CREATE TABLE IF NOT EXISTS chat_imports (
    id           TEXT PRIMARY KEY,
    project_id   TEXT NOT NULL,
    source       TEXT NOT NULL,  -- 'chatgpt', 'claude', 'gemini', 'cursor', 'markdown', 'json'
    filename     TEXT NOT NULL DEFAULT '',
    raw_content  TEXT NOT NULL,
    processed    INTEGER NOT NULL DEFAULT 0,
    memories_created INTEGER NOT NULL DEFAULT 0,
    decisions_extracted INTEGER NOT NULL DEFAULT 0,
    features_extracted INTEGER NOT NULL DEFAULT 0,
    bugs_extracted INTEGER NOT NULL DEFAULT 0,
    created_at   INTEGER NOT NULL,
    processed_at INTEGER,

    FOREIGN KEY (project_id) REFERENCES projects(id)
);

CREATE INDEX IF NOT EXISTS idx_chat_imports_project
    ON chat_imports(project_id, created_at DESC);
