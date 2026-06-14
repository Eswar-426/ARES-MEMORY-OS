-- Knowledge Extraction Engine (Week 24)
-- Stores intermediate KnowledgeCandidates extracted from git commits.

CREATE TABLE IF NOT EXISTS knowledge_candidates (
    id TEXT PRIMARY KEY,
    knowledge_type TEXT NOT NULL CHECK(knowledge_type IN ('decision', 'bug', 'architecture', 'experiment')),
    confidence REAL NOT NULL,
    reasoning TEXT NOT NULL,
    content TEXT NOT NULL,
    title TEXT NOT NULL,
    source_commit TEXT NOT NULL,
    affected_files TEXT, -- JSON array of file paths
    persisted INTEGER NOT NULL DEFAULT 0,
    project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
    extracted_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_kc_source_commit ON knowledge_candidates(source_commit);
CREATE INDEX IF NOT EXISTS idx_kc_knowledge_type ON knowledge_candidates(knowledge_type);
CREATE INDEX IF NOT EXISTS idx_kc_project_id ON knowledge_candidates(project_id);
CREATE INDEX IF NOT EXISTS idx_kc_confidence ON knowledge_candidates(confidence);
