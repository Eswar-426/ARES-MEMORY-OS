CREATE TABLE IF NOT EXISTS agent_sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    ended_at INTEGER NOT NULL,
    tool_calls TEXT NOT NULL DEFAULT '[]',
    summary TEXT NOT NULL DEFAULT '',
    files_touched TEXT NOT NULL DEFAULT '[]',
    decisions_referenced TEXT NOT NULL DEFAULT '[]',
    queries_asked TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE INDEX IF NOT EXISTS idx_agent_sessions_project ON agent_sessions(project_id, ended_at DESC);
