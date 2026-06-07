-- V3__decisions.sql
-- Decision DNA extended schema.

CREATE TABLE IF NOT EXISTS decisions (
  id                 TEXT PRIMARY KEY,
  project_id         TEXT NOT NULL REFERENCES projects(id),
  memory_id          TEXT NOT NULL REFERENCES memories(id),

  decision_text      TEXT NOT NULL,
  reason             TEXT NOT NULL,
  status             TEXT NOT NULL DEFAULT 'accepted'
                     CHECK(status IN ('proposed','accepted','rejected','superseded','deprecated')),
  confidence         REAL NOT NULL DEFAULT 1.0
                     CHECK(confidence >= 0.0 AND confidence <= 1.0),

  alternatives       TEXT NOT NULL DEFAULT '[]',
  risks              TEXT NOT NULL DEFAULT '[]',
  context_snapshot   TEXT NOT NULL DEFAULT '{}',
  future_impact      TEXT NOT NULL DEFAULT '{}',

  files_impacted     TEXT NOT NULL DEFAULT '[]',
  services_impacted  TEXT NOT NULL DEFAULT '[]',

  supersedes         TEXT NOT NULL DEFAULT '[]',
  superseded_by      TEXT REFERENCES decisions(id),

  decided_by         TEXT NOT NULL DEFAULT '',
  discussed_in       TEXT NOT NULL DEFAULT '[]',

  review_due_at      INTEGER,
  last_reviewed_at   INTEGER,

  created_at         INTEGER NOT NULL,
  updated_at         INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_decisions_project
  ON decisions(project_id);

CREATE INDEX IF NOT EXISTS idx_decisions_status
  ON decisions(project_id, status);

CREATE INDEX IF NOT EXISTS idx_decisions_review
  ON decisions(review_due_at)
  WHERE review_due_at IS NOT NULL AND superseded_by IS NULL;

CREATE INDEX IF NOT EXISTS idx_decisions_active
  ON decisions(project_id, status)
  WHERE status = 'accepted';

-- Reasoning chain steps (ordered)
CREATE TABLE IF NOT EXISTS decision_reasoning (
  id           TEXT PRIMARY KEY,
  decision_id  TEXT NOT NULL REFERENCES decisions(id) ON DELETE CASCADE,
  step_order   INTEGER NOT NULL,
  observation  TEXT NOT NULL,
  inference    TEXT NOT NULL,
  confidence   REAL NOT NULL DEFAULT 1.0,
  created_at   INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reasoning_decision
  ON decision_reasoning(decision_id, step_order);
