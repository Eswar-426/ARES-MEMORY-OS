-- Indexes for EvidenceService queries used by intelligence generators
CREATE INDEX IF NOT EXISTS idx_graph_nodes_path_like ON graph_nodes(file_path);
CREATE INDEX IF NOT EXISTS idx_graph_edges_contains_source ON graph_edges(from_node_id, edge_type) WHERE edge_type = 'contains';
CREATE INDEX IF NOT EXISTS idx_graph_edges_contains_target ON graph_edges(to_node_id, edge_type) WHERE edge_type = 'contains';
CREATE INDEX IF NOT EXISTS idx_graph_edges_depends_source ON graph_edges(from_node_id, edge_type) WHERE edge_type = 'depends_on';
CREATE INDEX IF NOT EXISTS idx_graph_edges_depends_target ON graph_edges(to_node_id, edge_type) WHERE edge_type = 'depends_on';
