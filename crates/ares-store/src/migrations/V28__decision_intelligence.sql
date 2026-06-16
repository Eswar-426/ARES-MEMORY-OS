-- V28: Decision Intelligence Engine Schema

CREATE TABLE IF NOT EXISTS decision_records (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    context TEXT NOT NULL,
    problem TEXT NOT NULL,
    chosen_option TEXT NOT NULL,
    rejected_options TEXT NOT NULL, -- JSON serialized Vec<DecisionAlternative>
    assumptions TEXT NOT NULL, -- JSON serialized Vec<String>
    consequences TEXT NOT NULL, -- JSON serialized Vec<DecisionConsequence>
    confidence TEXT NOT NULL,
    owner TEXT,
    approval_status TEXT NOT NULL,
    approved_by TEXT,
    approved_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_decision_records_status ON decision_records(approval_status);
CREATE INDEX IF NOT EXISTS idx_decision_records_owner ON decision_records(owner);

CREATE TABLE IF NOT EXISTS decision_links (
    id TEXT PRIMARY KEY,
    source_decision_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    target_type TEXT NOT NULL,
    relationship TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    created_by TEXT,
    FOREIGN KEY(source_decision_id) REFERENCES decision_records(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_decision_links_source ON decision_links(source_decision_id);
CREATE INDEX IF NOT EXISTS idx_decision_links_target ON decision_links(target_id, target_type);

CREATE TABLE IF NOT EXISTS decision_revisions (
    id TEXT PRIMARY KEY,
    decision_id TEXT NOT NULL,
    changed_by TEXT,
    change_reason TEXT,
    diff_payload TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(decision_id) REFERENCES decision_records(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_decision_revisions_decision ON decision_revisions(decision_id);

CREATE TABLE IF NOT EXISTS decision_evidence (
    id TEXT PRIMARY KEY,
    decision_id TEXT NOT NULL,
    source TEXT NOT NULL,
    reference_url TEXT NOT NULL,
    description TEXT NOT NULL,
    confidence_score REAL NOT NULL,
    FOREIGN KEY(decision_id) REFERENCES decision_records(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_decision_evidence_decision ON decision_evidence(decision_id);

ALTER TABLE decision_outcomes RENAME TO decision_dna_outcomes;

CREATE TABLE IF NOT EXISTS decision_outcomes (
    id TEXT PRIMARY KEY,
    decision_id TEXT NOT NULL,
    observed_at INTEGER NOT NULL,
    description TEXT NOT NULL,
    outcome_type TEXT NOT NULL,
    success_score REAL,
    FOREIGN KEY(decision_id) REFERENCES decision_records(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_decision_outcomes_decision ON decision_outcomes(decision_id);

CREATE TABLE IF NOT EXISTS decision_health_snapshots (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    snapshot_time INTEGER NOT NULL,
    total_decisions INTEGER NOT NULL,
    approved_decisions INTEGER NOT NULL,
    decisions_with_evidence INTEGER NOT NULL,
    decisions_with_consequences INTEGER NOT NULL,
    decisions_without_owner INTEGER NOT NULL,
    health_score REAL NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_decision_health_snapshots_project ON decision_health_snapshots(project_id);
