-- V45__expand_evolution_node_types.sql
-- Expand graph_nodes node_type constraint to include 'evolution_event'
-- Expand graph_edges edge_type constraint to include 'evolves', 'drifts' and source 'evolution_engine'

PRAGMA foreign_keys=off;

CREATE TABLE IF NOT EXISTS graph_nodes_new (
  id            TEXT PRIMARY KEY,
  project_id    TEXT NOT NULL REFERENCES projects(id),
  node_type     TEXT NOT NULL
                CHECK(node_type IN (
                  'project','file','function','method','class','struct','enum','trait','interface',
                  'module','service','decision','feature',
                  'bug','concept','tag','requirement','architecture','evidence','owner','repository',
                  'alternative','assumption','risk','folder','person',
                  'commit','branch','release','evolution_event'
                )),
  label         TEXT NOT NULL,
  properties    TEXT NOT NULL,
  file_path     TEXT,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  deleted_at    INTEGER
);

INSERT INTO graph_nodes_new SELECT * FROM graph_nodes;
DROP TABLE graph_nodes;
ALTER TABLE graph_nodes_new RENAME TO graph_nodes;

CREATE INDEX IF NOT EXISTS idx_nodes_project ON graph_nodes(project_id, node_type);
CREATE INDEX IF NOT EXISTS idx_nodes_type ON graph_nodes(node_type);
CREATE INDEX IF NOT EXISTS idx_nodes_deleted ON graph_nodes(deleted_at) WHERE deleted_at IS NULL;

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
                  'supported_by','validated_by','contributed_to',
                  'maintains','touches','authored_by','released_in',
                  'evolves','drifts'
                )),
  weight        REAL NOT NULL DEFAULT 1.0,
  confidence    REAL NOT NULL DEFAULT 1.0,
  source        TEXT NOT NULL DEFAULT 'scanner'
                CHECK(source IN ('human','scanner','agent','inference','git_commits','git_blame','git_releases','git_branches','codeowners', 'evolution_engine')),
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

PRAGMA foreign_keys=on;
