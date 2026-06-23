use crate::agent::{AgentType, BenchmarkMetrics};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub task: String,
    pub repository: String,
    pub timestamp: String,
    pub results: HashMap<AgentType, BenchmarkMetrics>,
}

impl BenchmarkReport {
    pub fn new(
        task: String,
        repository: String,
        results: HashMap<AgentType, BenchmarkMetrics>,
    ) -> Self {
        Self {
            task,
            repository,
            timestamp: chrono::Utc::now().to_rfc3339(),
            results,
        }
    }

    /// Generate a Markdown string report
    pub fn to_markdown(&self) -> String {
        let baseline = self
            .results
            .get(&AgentType::Baseline)
            .cloned()
            .unwrap_or_default();
        let ares = self
            .results
            .get(&AgentType::Ares)
            .cloned()
            .unwrap_or_default();

        // Calculate improvements (Baseline vs ARES)
        let token_diff = if baseline.total_tokens > 0 {
            100.0 - (ares.total_tokens as f64 / baseline.total_tokens as f64 * 100.0)
        } else {
            0.0
        };

        let file_read_diff = if baseline.search_depth > 0 {
            100.0 - (ares.search_depth as f64 / baseline.search_depth as f64 * 100.0)
        } else {
            0.0
        };

        let time_diff = if baseline.time_elapsed_secs > 0.0 {
            100.0 - (ares.time_elapsed_secs / baseline.time_elapsed_secs * 100.0)
        } else {
            0.0
        };

        let success_diff = ares.success_score - baseline.success_score;

        let mut output = "# ARES Benchmark Report\n\n".to_string();
        output.push_str(&format!("**Task**: {}\n", self.task));
        output.push_str(&format!("**Repository**: {}\n", self.repository));
        output.push_str(&format!("**Timestamp**: {}\n\n", self.timestamp));

        // Output each agent's stats
        let order = vec![
            AgentType::Baseline,
            AgentType::ContextDump,
            AgentType::Ares,
            AgentType::ContextDumpAndAres,
        ];

        for agent in order {
            if let Some(metrics) = self.results.get(&agent) {
                output.push_str(&format!("### {}\n", agent.as_str()));
                output.push_str("---------\n");
                output.push_str(&format!("Tokens: {}\n", metrics.total_tokens));
                output.push_str(&format!("Cost: ${:.4}\n", metrics.provider_cost_usd));
                output.push_str(&format!(
                    "Search Depth (Files Read): {}\n",
                    metrics.search_depth
                ));
                output.push_str(&format!("Time: {:.1} sec\n", metrics.time_elapsed_secs));
                output.push_str(&format!("Success Score: {:.1}%\n", metrics.success_score));
                output.push_str(&format!(
                    "Repeated Failure: {}\n\n",
                    metrics.repeated_failure
                ));
            }
        }

        output.push_str("### Improvement (ARES vs Baseline)\n");
        output.push_str("-----------\n");
        output.push_str(&format!("{:.1}% fewer tokens\n", token_diff));
        output.push_str(&format!("{:.1}% fewer file reads\n", file_read_diff));
        output.push_str(&format!("{:.1}% faster\n", time_diff));
        output.push_str(&format!("{:.1}% higher success\n", success_diff));

        output
    }

    /// Generate a JSON string report
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}
