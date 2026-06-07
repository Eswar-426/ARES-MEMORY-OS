-- V2__memories.sql
-- Base memory table with versioning + FTS5 virtual table.

CREATE TABLE IF NOT EXISTS memories (
  id           TEXT PRIMARY KEY,
  project_id   TEXT NOT NULL REFERENCES projects(id),
  memory_type  TEXT NOT NULL
               CHECK(memory_type IN (
                 'project','feature','bug','decision',
                 'architecture','agent','team','workflow'
               )),
  title        TEXT NOT NULL,
  content      TEXT NOT NULL,
  status       TEXT NOT NULL DEFAULT 'active'
               CHECK(status IN ('active','deprecated','archived')),
  version      INTEGER NOT NULL DEFAULT 1,
  parent_id    TEXT REFERENCES memories(id),
  confidence   REAL NOT NULL DEFAULT 1.0
               CHECK(confidence >= 0.0 AND confidence <= 1.0),
  source       TEXT NOT NULL DEFAULT 'human'
               CHECK(source IN ('human','scanner','agent','inference')),
  ai_assisted  INTEGER NOT NULL DEFAULT 0
               CHECK(ai_assisted IN (0,1)),
  created_at   INTEGER NOT NULL,
  updated_at   INTEGER NOT NULL,
  deleted_at   INTEGER
);

CREATE INDEX IF NOT EXISTS idx_memories_project
  ON memories(project_id, memory_type);

CREATE INDEX IF NOT EXISTS idx_memories_updated
  ON memories(project_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_memories_parent
  ON memories(parent_id)
  WHERE parent_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_memories_active
  ON memories(project_id, memory_type, status)
  WHERE deleted_at IS NULL;

-- FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
  memory_id UNINDEXED,
  title,
  content,
  tokenize = 'porter unicode61'
);

-- Auto-sync triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS memories_fts_insert
  AFTER INSERT ON memories
BEGIN
  INSERT INTO memories_fts(memory_id, title, content)
  VALUES (new.id, new.title, new.content);
END;

CREATE TRIGGER IF NOT EXISTS memories_fts_update
  AFTER UPDATE ON memories
BEGIN
  DELETE FROM memories_fts WHERE memory_id = old.id;
  INSERT INTO memories_fts(memory_id, title, content)
  VALUES (new.id, new.title, new.content);
END;

CREATE TRIGGER IF NOT EXISTS memories_fts_delete
  AFTER DELETE ON memories
BEGIN
  DELETE FROM memories_fts WHERE memory_id = old.id;
END;
