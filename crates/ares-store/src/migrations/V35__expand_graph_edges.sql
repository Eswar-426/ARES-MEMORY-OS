-- V35__expand_graph_edges.sql
-- Expand graph_edges edge_type constraint to include ALL edge types from EdgeType::as_str()

PRAGMA foreign_keys=off;

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
                  'contradicts','uses','derived_from',
                  'contains','contained_in',
                  'invokes','constructs','references',
                  'resolved_to','uses_module','uses_trait',
                  'constrains','has_risk','has_assumption',
                  'drives','satisfies','owned_by',
                  'supported_by','validated_by'
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
