-- V4__graph.sql
-- Knowledge graph stored as adjacency list.
-- PostgreSQL-compatible graph traversal via recursive CTEs.

CREATE TABLE IF NOT EXISTS graph_nodes (
  id           TEXT PRIMARY KEY,
  project_id   TEXT NOT NULL REFERENCES projects(id),
  node_type    TEXT NOT NULL
               CHECK(node_type IN (
                 'project','file','function','class',
                 'module','service','decision','feature',
                 'bug','concept','tag'
               )),
  label        TEXT NOT NULL,
  properties   TEXT NOT NULL DEFAULT '{}',
  file_path    TEXT,
  created_at   INTEGER NOT NULL,
  updated_at   INTEGER NOT NULL,
  deleted_at   INTEGER
);

CREATE INDEX IF NOT EXISTS idx_nodes_project
  ON graph_nodes(project_id, node_type);

CREATE INDEX IF NOT EXISTS idx_nodes_file
  ON graph_nodes(file_path)
  WHERE file_path IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_nodes_active
  ON graph_nodes(project_id, node_type)
  WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS graph_edges (
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
                  'contradicts'
                )),
  weight        REAL NOT NULL DEFAULT 1.0,
  confidence    REAL NOT NULL DEFAULT 1.0,
  source        TEXT NOT NULL DEFAULT 'scanner'
                CHECK(source IN ('human','scanner','agent','inference')),
  valid_from    INTEGER NOT NULL,
  valid_until   INTEGER,
  created_at    INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_edges_from
  ON graph_edges(from_node_id, edge_type)
  WHERE valid_until IS NULL;

CREATE INDEX IF NOT EXISTS idx_edges_to
  ON graph_edges(to_node_id, edge_type)
  WHERE valid_until IS NULL;

CREATE INDEX IF NOT EXISTS idx_edges_project
  ON graph_edges(project_id, edge_type)
  WHERE valid_until IS NULL;

-- Unique active edge: only one active edge per (from, to, type) triple
CREATE UNIQUE INDEX IF NOT EXISTS idx_edges_unique_active
  ON graph_edges(from_node_id, to_node_id, edge_type)
  WHERE valid_until IS NULL;
