-- V55__expand_gap_node_types.sql
PRAGMA foreign_keys=off;

CREATE TABLE IF NOT EXISTS graph_nodes_new (
  id            TEXT PRIMARY KEY,
  project_id    TEXT NOT NULL REFERENCES projects(id),
  node_type     TEXT NOT NULL
                CHECK(node_type IN (
                  'project','file','function','method','class','struct','enum','trait','interface',
                  'module','service','decision','feature',
                  'bug','concept','tag','requirement','architecture','evidence','owner','repository',
                  'alternative','assumption','risk','folder','person','team',
                  'commit','branch','release','evolution_event',
                  'test','runtime_signal','outcome','review_trigger',
                  'gap','root_cause','resolution','knowledge_gap'
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

PRAGMA foreign_keys=on;
