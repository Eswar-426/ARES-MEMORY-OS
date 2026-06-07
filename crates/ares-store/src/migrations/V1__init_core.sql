-- V1__init_core.sql
-- Core project metadata and bootstrap table.
-- All other tables reference projects(id).

CREATE TABLE IF NOT EXISTS ares_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
  id               TEXT PRIMARY KEY,
  name             TEXT NOT NULL,
  description      TEXT NOT NULL DEFAULT '',
  root_path        TEXT NOT NULL UNIQUE,
  primary_language TEXT NOT NULL DEFAULT '',
  domain           TEXT NOT NULL DEFAULT '',
  maturity         TEXT NOT NULL DEFAULT 'greenfield'
                   CHECK(maturity IN ('greenfield','growth','mature','legacy')),
  created_at       INTEGER NOT NULL,
  updated_at       INTEGER NOT NULL,
  deleted_at       INTEGER
);
