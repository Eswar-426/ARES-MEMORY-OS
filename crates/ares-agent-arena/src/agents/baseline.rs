use super::AgentRunner;
use crate::models::{AgentRunResult, AgentType, ArenaTask};
use anyhow::Result;
use ares_core::types::pagination::Pagination;
use ares_core::ProjectId;
use ares_store::repositories::graph::SqliteGraphRepository;
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

/// Common English words that should be skipped during keyword search
const SKIP_WORDS: &[&str] = &[
    "explain", "trace", "find", "locate", "summarize", "identify", "provide",
    "describe", "analyze", "list", "show", "impact", "analysis", "all",
    "the", "and", "for", "how", "what", "where", "which", "that", "from",
    "with", "this", "have", "does", "about", "into", "would", "could",
    "should", "a", "an", "of", "in", "to", "by", "on", "is", "are", "be",
    "internal", "external", "downstream", "comprehensive", "high", "level",
    "utilized", "coordinates", "execution", "extraction", "interfaces",
    "generating", "repository", "components", "steps", "sequence", "initial",
    "changes", "affected", "responsible", "dependencies",
];

pub struct BaselineAgent {
    pub graph_repo: Arc<SqliteGraphRepository>,
    pub project_id: ProjectId,
}

#[async_trait]
impl AgentRunner for BaselineAgent {
    async fn run(&self, task: &ArenaTask) -> Result<AgentRunResult> {
        let start = Instant::now();

        // Extract meaningful keywords from title and description
        let combined = format!("{} {}", task.title, task.description);
        let keywords: Vec<&str> = combined
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_'))
            .filter(|w| w.len() >= 3)
            .filter(|w| !SKIP_WORDS.contains(&w.to_lowercase().as_str()))
            .collect();

        let pagination = Pagination { page: 1, page_size: 50 };
        let mut all_files = HashSet::new();
        let mut all_components = HashSet::new();

        // Search with each keyword (top 3 by length)
        let mut unique_keywords: Vec<&str> = keywords.clone();
        unique_keywords.sort_by(|a, b| b.len().cmp(&a.len()));
        unique_keywords.dedup();
        unique_keywords.truncate(3);

        for keyword in &unique_keywords {
            if let Ok(page) = self.graph_repo.list_nodes_paginated(
                &self.project_id,
                None,
                Some(keyword),
                &pagination,
            ) {
                for node in &page.items {
                    if let Some(fp) = &node.file_path {
                        all_files.insert(fp.clone());
                    }
                    if matches!(node.node_type,
                        ares_core::NodeType::Struct
                        | ares_core::NodeType::Function
                        | ares_core::NodeType::Class
                        | ares_core::NodeType::Trait
                        | ares_core::NodeType::Enum
                    ) {
                        all_components.insert(node.label.clone());
                    }
                }
            }
        }

        let latency_ms = start.elapsed().as_millis() as u64;
        let response = format!("Found {} files via keyword search ({:?}).", all_files.len(), unique_keywords);
        let retrieved_files: Vec<String> = all_files.into_iter().collect();
        let retrieved_components: Vec<String> = all_components.into_iter().collect();

        Ok(AgentRunResult {
            task_id: task.id.clone(),
            agent_type: AgentType::Baseline,
            response,
            latency_ms,
            context_nodes_used: retrieved_files.len() + retrieved_components.len(),
            retrieved_files,
            retrieved_components,
            precision_score: 0.0,
            recall_score: 0.0,
            confidence_score: 0.0,
            overall_score: 0.0,
            graph_coverage: 0.1, // Baseline uses keyword search, poor coverage
            context_efficiency: 0.1, // Keyword search efficiency is low
        })
    }
}
