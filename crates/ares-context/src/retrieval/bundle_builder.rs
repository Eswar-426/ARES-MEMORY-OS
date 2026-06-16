use crate::models::{ContextBundle, DependencyTrace, FileExplanation, ImpactReport, RepositoryInsight};
use ares_core::GraphNode;

pub struct BundleBuilder {
    pub bundle: ContextBundle,
}

impl BundleBuilder {
    pub fn new() -> Self {
        Self {
            bundle: ContextBundle::default(),
        }
    }

    pub fn set_query(&mut self, query: String) -> &mut Self {
        self.bundle.query = query;
        self
    }

    pub fn set_intent(&mut self, intent: crate::query::intent::QueryIntent) -> &mut Self {
        self.bundle.intent = intent;
        self
    }

    pub fn add_target_nodes(&mut self, nodes: Vec<GraphNode>) -> &mut Self {
        self.bundle.target_nodes.extend(nodes);
        self
    }

    pub fn set_ranked_nodes(&mut self, nodes: Vec<GraphNode>) -> &mut Self {
        self.bundle.ranked_nodes = nodes;
        self
    }

    pub fn add_impact_report(&mut self, report: ImpactReport) -> &mut Self {
        self.bundle.impact_reports.push(report);
        self
    }

    pub fn add_dependency_trace(&mut self, trace: DependencyTrace) -> &mut Self {
        self.bundle.dependency_traces.push(trace);
        self
    }

    pub fn add_explanation(&mut self, explanation: FileExplanation) -> &mut Self {
        self.bundle.explanations.push(explanation);
        self
    }

    pub fn add_insight(&mut self, insight: RepositoryInsight) -> &mut Self {
        self.bundle.repository_insights.push(insight);
        self
    }

    pub fn set_metrics(&mut self, metrics: crate::models::metrics::ContextMetrics) -> &mut Self {
        self.bundle.metrics = metrics;
        self
    }

    pub fn set_reachable_nodes(&mut self, count: usize) -> &mut Self {
        self.bundle.reachable_nodes = count;
        self
    }

    pub fn build(self) -> ContextBundle {
        self.bundle
    }
}
