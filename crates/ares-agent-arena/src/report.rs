use crate::models::AgentRunResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaReport {
    pub task_id: String,
    pub baseline: Option<AgentRunResult>,
    pub context: Option<AgentRunResult>,
    pub enhanced: Option<AgentRunResult>,
    pub planner: Option<AgentRunResult>,
}

pub struct ReportGenerator {
    arena_dir: PathBuf,
}

impl ReportGenerator {
    pub fn new(workspace_root: &str) -> Self {
        let mut path = PathBuf::from(workspace_root);
        path.push("artifacts");
        path.push("arena");
        fs::create_dir_all(&path).unwrap_or_default();
        Self { arena_dir: path }
    }

    pub fn save_json(&self, reports: &[ArenaReport]) -> Result<()> {
        let filename = "arena_reports.json";
        let mut path = self.arena_dir.clone();
        path.push(filename);
        
        let json = serde_json::to_string_pretty(reports)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn generate_markdown(&self, reports: &[ArenaReport]) -> Result<()> {
        let mut path = self.arena_dir.clone();
        path.push("latest_report.md");

        let mut md = String::new();
        md.push_str("# ARES Agent Arena Benchmark Report\n\n");

        for report in reports {
            md.push_str(&format!("## Task: {}\n\n", report.task_id));

            let mut best_agent = String::from("None");
            let mut best_precision = -1.0;

            let mut process_agent = |name: &str, res: &Option<AgentRunResult>, md_str: &mut String, best_a: &mut String, best_p: &mut f32| {
                if let Some(r) = res {
                    md_str.push_str(&format!("### {}\n", name));
                    md_str.push_str(&format!("- **Precision**: {:.2}\n", r.precision_score));
                    md_str.push_str(&format!("- **Recall**: {:.2}\n", r.recall_score));
                    md_str.push_str(&format!("- **Latency**: {} ms\n", r.latency_ms));
                    md_str.push_str(&format!("- **Nodes Used**: {}\n", r.context_nodes_used));
                    md_str.push_str("\n");

                    if r.precision_score > *best_p {
                        *best_p = r.precision_score;
                        *best_a = name.to_string();
                    }
                }
            };

            process_agent("Baseline Agent", &report.baseline, &mut md, &mut best_agent, &mut best_precision);
            process_agent("Context Aware Agent", &report.context, &mut md, &mut best_agent, &mut best_precision);
            process_agent("Enhanced Context Agent", &report.enhanced, &mut md, &mut best_agent, &mut best_precision);
            process_agent("Planner Agent", &report.planner, &mut md, &mut best_agent, &mut best_precision);

            md.push_str(&format!("**Winner**: {}\n\n", best_agent));
            md.push_str("---\n\n");
        }

        fs::write(path, md)?;
        Ok(())
    }
}
