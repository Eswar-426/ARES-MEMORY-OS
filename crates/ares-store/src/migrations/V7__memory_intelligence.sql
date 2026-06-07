-- V7__memory_intelligence.sql
-- Schema enhancements for the Memory Intelligence Engine (Week 3)

PRAGMA foreign_keys=off;

-- 1. Add Importance Level to Memories
ALTER TABLE memories ADD COLUMN importance TEXT NOT NULL DEFAULT 'medium'
  CHECK(importance IN ('critical','high','medium','low'));

-- 2. Expand Edge Types to include 'derived_from'
CREATE TABLE IF NOT EXISTS graph_edges_new (
  id            TEXT PRIMARY KEY,
  project_id    TEXT NOT NULL REFERENCES projects(id),
  from_node_id  TEXT NOT NULL REFERENCES graph_nodes(id),
  to_node_id    TEXT NOT NULL REFERENCES graph_nodes(id),
  edge_type     TEXT NOT NULL
                CHECK(edge_type IN (
                  'imports','defines','calls','extends',
                  'depends_on','implements','caused','fixed_by',
                  'supersedes','motivated_by','impacts','owns',
                  'authored','related_to','temporal_follows',
                  'contradicts','uses','derived_from'
                )),
  weight        REAL NOT NULL DEFAULT 1.0,
  confidence    REAL NOT NULL DEFAULT 1.0,
  source        TEXT NOT NULL DEFAULT 'scanner'
                CHECK(source IN ('human','scanner','agent','inference')),
  valid_from    INTEGER NOT NULL,
  valid_until   INTEGER,
  created_at    INTEGER NOT NULL
);

INSERT INTO graph_edges_new SELECT * FROM graph_edges;
DROP TABLE graph_edges;
ALTER TABLE graph_edges_new RENAME TO graph_edges;

CREATE INDEX IF NOT EXISTS idx_edges_from
  ON graph_edges(from_node_id, edge_type)
  WHERE valid_until IS NULL;

CREATE INDEX IF NOT EXISTS idx_edges_to
  ON graph_edges(to_node_id, edge_type)
  WHERE valid_until IS NULL;

CREATE INDEX IF NOT EXISTS idx_edges_project
  ON graph_edges(project_id, edge_type)
  WHERE valid_until IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_edges_unique_active
  ON graph_edges(from_node_id, to_node_id, edge_type)
  WHERE valid_until IS NULL;

-- 3. Memory Access Log (Frequency & Recency Tracking)
CREATE TABLE IF NOT EXISTS memory_access_log (
  id          TEXT PRIMARY KEY,
  memory_id   TEXT NOT NULL REFERENCES memories(id),
  project_id  TEXT NOT NULL REFERENCES projects(id),
  accessed_at INTEGER NOT NULL,
  context     TEXT NOT NULL DEFAULT 'query'
              CHECK(context IN ('query', 'background_scan', 'context_assembly', 'retrieval'))
);

CREATE INDEX IF NOT EXISTS idx_memory_access
  ON memory_access_log(memory_id, accessed_at DESC);

-- 4. Ranking Cache (Precomputed rankings)
CREATE TABLE IF NOT EXISTS ranking_cache (
  memory_id   TEXT PRIMARY KEY REFERENCES memories(id),
  project_id  TEXT NOT NULL REFERENCES projects(id),
  score       REAL NOT NULL,
  updated_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ranking_score
  ON ranking_cache(project_id, score DESC);

-- 5. Contradiction Records
CREATE TABLE IF NOT EXISTS contradiction_records (
  id          TEXT PRIMARY KEY,
  project_id  TEXT NOT NULL REFERENCES projects(id),
  source_id   TEXT NOT NULL REFERENCES graph_nodes(id),
  target_id   TEXT NOT NULL REFERENCES graph_nodes(id),
  reason      TEXT NOT NULL,
  confidence  REAL NOT NULL DEFAULT 1.0,
  created_at  INTEGER NOT NULL,
  resolved_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_contradictions_project
  ON contradiction_records(project_id, resolved_at);

PRAGMA foreign_keys=on;
