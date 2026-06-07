-- V5__events_scanner.sql
-- Internal event log (append-only) and scanner state tracking.

-- Events table — strictly append-only, no updates
CREATE TABLE IF NOT EXISTS events (
  id           TEXT PRIMARY KEY,
  project_id   TEXT REFERENCES projects(id),
  event_type   TEXT NOT NULL,
  payload      TEXT NOT NULL DEFAULT '{}',
  source       TEXT NOT NULL DEFAULT 'agent'
               CHECK(source IN ('agent','scanner','user','mcp')),
  created_at   INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_events_project
  ON events(project_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_events_type
  ON events(event_type, created_at DESC);

-- Prevent UPDATE and DELETE on events (append-only guarantee)
CREATE TRIGGER IF NOT EXISTS events_no_update
  BEFORE UPDATE ON events
BEGIN
  SELECT RAISE(ABORT, 'events table is append-only: UPDATE not permitted');
END;

CREATE TRIGGER IF NOT EXISTS events_no_delete
  BEFORE DELETE ON events
BEGIN
  SELECT RAISE(ABORT, 'events table is append-only: DELETE not permitted');
END;

-- Scanner state — tracks last-seen hash per file
CREATE TABLE IF NOT EXISTS scan_state (
  id           TEXT PRIMARY KEY,
  project_id   TEXT NOT NULL REFERENCES projects(id),
  file_path    TEXT NOT NULL,
  file_hash    TEXT NOT NULL,
  last_scanned INTEGER NOT NULL,
  node_ids     TEXT NOT NULL DEFAULT '[]',
  UNIQUE(project_id, file_path)
);

CREATE INDEX IF NOT EXISTS idx_scan_project
  ON scan_state(project_id);

-- Scanner run history
CREATE TABLE IF NOT EXISTS scan_runs (
  id           TEXT PRIMARY KEY,
  project_id   TEXT NOT NULL REFERENCES projects(id),
  run_type     TEXT NOT NULL CHECK(run_type IN ('full','incremental','watch')),
  status       TEXT NOT NULL CHECK(status IN ('running','completed','failed','cancelled')),
  files_total  INTEGER NOT NULL DEFAULT 0,
  files_parsed INTEGER NOT NULL DEFAULT 0,
  files_failed INTEGER NOT NULL DEFAULT 0,
  started_at   INTEGER NOT NULL,
  completed_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_scan_runs_project
  ON scan_runs(project_id, started_at DESC);
