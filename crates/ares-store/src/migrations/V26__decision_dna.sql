-- V26__decision_dna.sql

-- Update existing decisions table to match new Decision DNA models
ALTER TABLE decisions ADD COLUMN title TEXT NOT NULL DEFAULT '';
ALTER TABLE decisions ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE decisions ADD COLUMN ai_assisted BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE decisions ADD COLUMN human_reviewed BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE decisions ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';

-- Outcomes
CREATE TABLE IF NOT EXISTS decision_outcomes (
    decision_id TEXT PRIMARY KEY REFERENCES decisions(id) ON DELETE CASCADE,
    success_score REAL NOT NULL,
    lessons_learned TEXT NOT NULL DEFAULT '[]',
    measured_at INTEGER NOT NULL
);

-- Requirements
CREATE TABLE IF NOT EXISTS requirements (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    priority TEXT NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

-- Provenance
CREATE TABLE IF NOT EXISTS decision_provenance (
    decision_id TEXT PRIMARY KEY REFERENCES decisions(id) ON DELETE CASCADE,
    source_type TEXT NOT NULL,
    author_id TEXT,
    created_by_agent TEXT,
    reviewed_by TEXT,
    confidence REAL NOT NULL,
    source_system TEXT NOT NULL,
    original_commit TEXT,
    pull_request_url TEXT,
    evidence_links TEXT NOT NULL DEFAULT '[]'
);

-- Audit Log
CREATE TABLE IF NOT EXISTS decision_audit_log (
    id TEXT PRIMARY KEY,
    decision_id TEXT NOT NULL REFERENCES decisions(id) ON DELETE CASCADE,
    previous_state TEXT NOT NULL,
    new_state TEXT NOT NULL,
    changed_by TEXT,
    changed_at INTEGER NOT NULL,
    reason TEXT NOT NULL
);
