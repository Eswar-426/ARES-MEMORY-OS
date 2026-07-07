-- V53: Drop redundant indexes that bloat the database
-- Root cause: table renames (entities->graph_nodes) and multiple migration
-- rounds created duplicate indexes. V52 compound indexes made older
-- single-column and partial indexes redundant.

-- graph_edges: 8 redundant (covered by idx_graph_edges_source_type / idx_graph_edges_target_type)
DROP INDEX IF EXISTS idx_edges_from;
DROP INDEX IF EXISTS idx_edges_to;
DROP INDEX IF EXISTS idx_graph_edges_source;
DROP INDEX IF EXISTS idx_graph_edges_target;
DROP INDEX IF EXISTS idx_graph_edges_contains_source;
DROP INDEX IF EXISTS idx_graph_edges_contains_target;
DROP INDEX IF EXISTS idx_graph_edges_depends_source;
DROP INDEX IF EXISTS idx_graph_edges_depends_target;

-- graph_nodes: 1 redundant (exact duplicate)
DROP INDEX IF EXISTS idx_nodes_type;

-- graph_entities: 2 redundant (exact duplicates from table rename)
DROP INDEX IF EXISTS idx_graph_entities_name;
DROP INDEX IF EXISTS idx_graph_entities_type;

-- graph_relationships: 3 redundant (exact duplicates from table rename)
DROP INDEX IF EXISTS idx_graph_rel_source;
DROP INDEX IF EXISTS idx_graph_rel_target;
DROP INDEX IF EXISTS idx_graph_rel_type;
