use crate::models::{ContextBundle, ContextPack, ContextBudget, RetrievalExplanation};
use chrono::Utc;
use std::collections::HashSet;

pub struct ContextPackBuilder {
    budget: ContextBudget,
}

impl ContextPackBuilder {
    pub fn new(budget: ContextBudget) -> Self {
        Self { budget }
    }

    pub fn build(&self, bundle: ContextBundle) -> ContextPack {
        // Enforce budgeting constraints
        let max_nodes = self.budget.max_nodes;
        let max_dependencies = self.budget.max_dependencies;
        let max_impact_entries = self.budget.max_impact_entries;

        // Truncate nodes
        let ranked_nodes = bundle.ranked_nodes.into_iter().take(max_nodes).collect::<Vec<_>>();
        
        let mut relevant_files = HashSet::new();
        for node in &ranked_nodes {
            if let Some(path) = &node.file_path {
                relevant_files.insert(path.clone());
            }
        }
        // Limit relevant files based on budget
        let relevant_files = relevant_files.into_iter().take(self.budget.max_files).collect::<Vec<_>>();

        // Truncate dependencies
        let mut dependency_trace = bundle.dependency_traces;
        if dependency_trace.len() > max_dependencies {
            dependency_trace.truncate(max_dependencies);
        }

        // Truncate impact reports
        let mut impact_analysis = bundle.impact_reports;
        if impact_analysis.len() > max_impact_entries {
            impact_analysis.truncate(max_impact_entries);
        }

        // We build a simple explanation based on top ranked nodes
        let selected_nodes = ranked_nodes.iter().map(|n| n.label.clone()).collect::<Vec<_>>();
        let ranking_reasons = if !selected_nodes.is_empty() {
            vec!["Matches query intent and graph centrality.".to_string()]
        } else {
            vec![]
        };

        let explanation = RetrievalExplanation {
            selected_nodes,
            ranking_reasons,
        };

        let token_estimate: usize = ranked_nodes.iter().map(|n| n.properties.to_string().len()).sum::<usize>() / 4;
        let nodes_selected = ranked_nodes.len();
        let context_efficiency = if token_estimate > 0 {
            nodes_selected as f64 / token_estimate as f64
        } else {
            0.0
        };

        let mut final_metrics = bundle.metrics;
        final_metrics.nodes_selected = nodes_selected;
        final_metrics.files_selected = relevant_files.len();
        final_metrics.token_estimate = token_estimate;
        final_metrics.avg_depth = 1.0; 
        final_metrics.max_depth = self.budget.max_depth;
        final_metrics.context_efficiency = context_efficiency;

        if let Ok(root) = std::env::current_dir() {
            let out_dir = root.join("artifacts").join("validation");
            let _ = std::fs::create_dir_all(&out_dir);
            let out_file = out_dir.join("context_metrics.json");
            let _ = std::fs::write(&out_file, serde_json::to_string_pretty(&final_metrics).unwrap_or_default());
        }

        ContextPack {
            query: bundle.query,
            intent: bundle.intent,
            summary: format!("ARES Context generated for intent."), // could be better summarized
            relevant_files,
            relevant_nodes: ranked_nodes,
            dependency_trace,
            impact_analysis,
            architecture_paths: vec![],
            memory_snippets: vec![], // TODO
            confidence_score: 0.9,   // In a real system, calculated from metrics
            generated_at: Utc::now(),
            retrieval_time_ms: final_metrics.retrieval_time_ms,
            retrieval_explanation: explanation,
            metrics: final_metrics,
        }
    }
}
