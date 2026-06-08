#[cfg(test)]
mod tests {
    use ares_agent::services::{
        architectural_analysis::ArchitecturalAnalysisEngine,
        graph_clustering::GraphClusteringEngine,
        impact_prediction::ImpactPredictionEngine,
        knowledge_graph_engine::KnowledgeGraphEngine,
        root_cause_engine::RootCauseEngine,
        graph_cache::GraphCache,
    };
    use ares_core::{AnalysisScope, EdgeType, GraphEdge, GraphNode, GraphStatistics, KnowledgeGraph, NodeId, NodeType, ProjectId};
    use std::sync::Arc;
    use std::time::Instant;

    fn generate_large_graph(node_count: usize, edge_count: usize) -> KnowledgeGraph {
        let pid = ProjectId::from("perf_test_project");
        let mut nodes = Vec::with_capacity(node_count);
        for i in 0..node_count {
            nodes.push(GraphNode {
                id: NodeId::from(format!("node_{}", i)),
                project_id: pid.clone(),
                node_type: NodeType::Module,
                label: format!("Module {}", i),
                properties: serde_json::json!({}),
                file_path: None,
                created_at: 0,
                updated_at: 0,
                deleted_at: None,
            });
        }

        let mut edges = Vec::with_capacity(edge_count);
        for i in 0..edge_count {
            let from = i % node_count;
            let to = (i + 1) % node_count;
            edges.push(GraphEdge {
                id: format!("edge_{}", i),
                project_id: pid.clone(),
                from_node_id: NodeId::from(format!("node_{}", from)),
                to_node_id: NodeId::from(format!("node_{}", to)),
                edge_type: EdgeType::DependsOn,
                weight: 1.0,
                confidence: 1.0,
                source: "perf_test".into(),
                valid_from: 0,
                valid_until: None,
                created_at: 0,
            });
        }

        KnowledgeGraph {
            nodes,
            edges,
            statistics: GraphStatistics {
                total_nodes: 0,
                total_edges: 0,
                density: 0.0,
                average_degree: 0.0,
                connected_components: 0,
            },
        }
    }

    #[test]
    fn test_large_scale_impact_prediction() {
        let kg = generate_large_graph(100_000, 1_000_000);
        let engine = ImpactPredictionEngine::new();
        
        let start = Instant::now();
        let _result = engine.predict_change_impact(&kg, &NodeId::from("node_0")).unwrap();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 500, "Impact prediction took too long: {}ms", duration.as_millis());
    }

    #[test]
    fn test_large_scale_root_cause() {
        let kg = generate_large_graph(10_000, 100_000);
        let engine = RootCauseEngine::new();
        
        let start = Instant::now();
        let _result = engine.find_root_cause(&kg, &NodeId::from("node_9999")).unwrap();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 500, "Root cause took too long: {}ms", duration.as_millis());
    }
}
