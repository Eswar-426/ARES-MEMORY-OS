-- V52: Update graph indexes for renamed tables

-- Edge Source Index
CREATE INDEX IF NOT EXISTS idx_graph_edges_source
ON graph_edges(from_node_id);

-- Edge Target Index
CREATE INDEX IF NOT EXISTS idx_graph_edges_target
ON graph_edges(to_node_id);

-- Edge Relationship Type Index
CREATE INDEX IF NOT EXISTS idx_graph_edges_type
ON graph_edges(edge_type);

-- Edge Source + Type Compound Index
CREATE INDEX IF NOT EXISTS idx_graph_edges_source_type
ON graph_edges(from_node_id, edge_type);

-- Edge Target + Type Compound Index
CREATE INDEX IF NOT EXISTS idx_graph_edges_target_type
ON graph_edges(to_node_id, edge_type);

-- Node Entity Type Index
CREATE INDEX IF NOT EXISTS idx_graph_nodes_type
ON graph_nodes(node_type);

-- Node Name Index
CREATE INDEX IF NOT EXISTS idx_graph_nodes_label
ON graph_nodes(label);
