-- V31__memory_evolution.sql
-- Phase 7: Memory Evolution Engine (Historical Tracking)

CREATE TABLE IF NOT EXISTS memory_revisions (
    revision_id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    change_type TEXT NOT NULL, -- Created, Updated, Superseded, Approved, Rejected, Archived
    changed_at INTEGER NOT NULL,
    changed_by TEXT,
    reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_memory_revisions_entity_id ON memory_revisions(entity_id);
CREATE INDEX IF NOT EXISTS idx_memory_revisions_changed_at ON memory_revisions(changed_at);
CREATE INDEX IF NOT EXISTS idx_memory_revisions_entity_type ON memory_revisions(entity_type);

CREATE TABLE IF NOT EXISTS memory_diffs (
    revision_id TEXT PRIMARY KEY REFERENCES memory_revisions(revision_id) ON DELETE CASCADE,
    before_state TEXT NOT NULL, -- JSON
    after_state TEXT NOT NULL   -- JSON
);

CREATE TABLE IF NOT EXISTS entity_supersession (
    supersession_id TEXT PRIMARY KEY,
    superseded_entity_id TEXT NOT NULL,
    superseding_entity_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    superseded_at INTEGER NOT NULL,
    reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_supersession_superseded ON entity_supersession(superseded_entity_id);
CREATE INDEX IF NOT EXISTS idx_supersession_superseding ON entity_supersession(superseding_entity_id);

CREATE TABLE IF NOT EXISTS memory_revision_events (
    event_id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL,
    processed_at INTEGER NOT NULL
);
