-- V14__graph_indexes.sql
-- Optimizes Graph Traversal with Adjacency Lookups

-- Edge Source Index
CREATE INDEX IF NOT EXISTS idx_graph_rel_source
ON graph_relationships(source_entity);

-- Edge Target Index
CREATE INDEX IF NOT EXISTS idx_graph_rel_target
ON graph_relationships(target_entity);

-- Edge Relationship Type Index
CREATE INDEX IF NOT EXISTS idx_graph_rel_type
ON graph_relationships(relationship_type);

-- Edge Source + Type Compound Index
CREATE INDEX IF NOT EXISTS idx_graph_rel_source_type
ON graph_relationships(source_entity, relationship_type);

-- Edge Target + Type Compound Index
CREATE INDEX IF NOT EXISTS idx_graph_rel_target_type
ON graph_relationships(target_entity, relationship_type);

-- Node Entity Type Index
CREATE INDEX IF NOT EXISTS idx_graph_entity_type
ON graph_entities(entity_type);

-- Node Name Index
CREATE INDEX IF NOT EXISTS idx_graph_entity_name
ON graph_entities(name);
