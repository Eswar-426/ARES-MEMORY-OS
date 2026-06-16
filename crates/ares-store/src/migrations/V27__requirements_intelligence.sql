-- ============================================================
-- V27: Requirement Intelligence Engine
-- Extends the basic requirements table from V26 into a
-- full memory-grade requirement tracking system.
-- ============================================================

-- Extend the existing requirements table (V26 created: id, title, description, priority, status, source, created_at)
ALTER TABLE requirements ADD COLUMN requirement_type TEXT NOT NULL DEFAULT 'functional';
ALTER TABLE requirements ADD COLUMN owner TEXT;
ALTER TABLE requirements ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE requirements ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
ALTER TABLE requirements ADD COLUMN project_id TEXT NOT NULL DEFAULT '';

-- Indexes on the extended columns
CREATE INDEX IF NOT EXISTS idx_requirements_project ON requirements(project_id);
CREATE INDEX IF NOT EXISTS idx_requirements_status ON requirements(status);
CREATE INDEX IF NOT EXISTS idx_requirements_type ON requirements(requirement_type);
CREATE INDEX IF NOT EXISTS idx_requirements_owner ON requirements(owner);

-- ============================================================
-- Requirement Links — THE single source of truth for all
-- requirement relationships. No relationship data lives on
-- the Requirement entity itself.
-- ============================================================
CREATE TABLE IF NOT EXISTS requirement_links (
    id TEXT PRIMARY KEY,
    source_requirement_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    target_type TEXT NOT NULL CHECK(target_type IN (
        'requirement', 'decision', 'architecture', 'code'
    )),
    relationship TEXT NOT NULL CHECK(relationship IN (
        'drives', 'implements', 'traces_to', 'validates',
        'depends_on', 'blocks', 'parent_of', 'child_of',
        'supersedes', 'derived_from'
    )),
    created_at INTEGER NOT NULL,
    created_by TEXT,
    UNIQUE(source_requirement_id, target_id, target_type, relationship)
);

CREATE INDEX IF NOT EXISTS idx_req_links_source
    ON requirement_links(source_requirement_id);
CREATE INDEX IF NOT EXISTS idx_req_links_target
    ON requirement_links(target_id, target_type);
CREATE INDEX IF NOT EXISTS idx_req_links_relationship
    ON requirement_links(relationship);

-- ============================================================
-- Requirement Revisions — change history (memory, not storage)
-- Tracks who/when/what/why for every change.
-- ============================================================
CREATE TABLE IF NOT EXISTS requirement_revisions (
    id TEXT PRIMARY KEY,
    requirement_id TEXT NOT NULL,
    revision_number INTEGER NOT NULL,
    previous_state TEXT NOT NULL,   -- JSON snapshot of requirement before change
    new_state TEXT NOT NULL,        -- JSON snapshot of requirement after change
    changed_fields TEXT NOT NULL,   -- JSON array of field names that changed
    changed_by TEXT,
    change_reason TEXT,
    created_at INTEGER NOT NULL,
    UNIQUE(requirement_id, revision_number)
);

CREATE INDEX IF NOT EXISTS idx_req_revisions_req
    ON requirement_revisions(requirement_id);

-- ============================================================
-- Requirement Health Snapshots — periodic health scoring
-- ============================================================
CREATE TABLE IF NOT EXISTS requirement_health_snapshots (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    snapshot_json TEXT NOT NULL,     -- Full RequirementHealth serialized
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_req_health_project
    ON requirement_health_snapshots(project_id);

-- ============================================================
-- Requirement Evidence — auditability and confidence
-- ============================================================
CREATE TABLE IF NOT EXISTS requirement_evidence (
    id TEXT PRIMARY KEY,
    requirement_id TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK(source_type IN (
        'jira', 'adr', 'rfc', 'commit', 'pr', 'issue', 'document', 'other'
    )),
    source_reference TEXT NOT NULL,
    confidence_score REAL NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_req_evidence_req
    ON requirement_evidence(requirement_id);
