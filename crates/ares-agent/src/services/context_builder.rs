use crate::services::context_intelligence::ContextAnalysis;
use crate::services::evolution_engine::EvolutionAnalysis;
use ares_core::{Decision, Memory, Project};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextCompressionLevel {
    Full,
    Compressed,
    ExecutiveSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudget {
    pub max_total_tokens: usize,
    pub compression_level: ContextCompressionLevel,
}

impl ContextBudget {
    pub fn budget_2k() -> Self {
        Self {
            max_total_tokens: 2000,
            compression_level: ContextCompressionLevel::ExecutiveSummary,
        }
    }
    pub fn budget_4k() -> Self {
        Self {
            max_total_tokens: 4000,
            compression_level: ContextCompressionLevel::Compressed,
        }
    }
    pub fn budget_8k() -> Self {
        Self {
            max_total_tokens: 8000,
            compression_level: ContextCompressionLevel::Full,
        }
    }
    pub fn budget_16k() -> Self {
        Self {
            max_total_tokens: 16000,
            compression_level: ContextCompressionLevel::Full,
        }
    }
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self::budget_8k()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningContext {
    pub memories: Vec<Memory>,
    pub decisions: Vec<Decision>,
    pub contradictions: Vec<String>,
    pub dependencies: Vec<String>,
    pub timeline: Option<EvolutionAnalysis>,
    pub confidence: f32,
    pub summary: String,
    pub estimated_tokens: usize,
}

pub struct ReasoningContextBuilder;

impl ReasoningContextBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReasoningContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ReasoningContextBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &self,
        project: &Project,
        query: &str,
        mut memories: Vec<Memory>,
        mut decisions: Vec<Decision>,
        context_analysis: ContextAnalysis,
        timeline: Option<EvolutionAnalysis>,
        budget: ContextBudget,
    ) -> ReasoningContext {
        // Deterministic ordering: highest relevance first, but for this mock we just use vector order.
        let mut snapshot_memories = Vec::new();
        let mut snapshot_decisions = Vec::new();

        let base_text = format!("{} {} {}", project.name, project.description, query);
        let mut total_tokens = self.estimate_tokens(&base_text);

        for mem in memories.drain(..) {
            let tokens = self.estimate_memory_tokens(&mem, &budget.compression_level);
            if total_tokens + tokens > budget.max_total_tokens {
                break;
            }
            total_tokens += tokens;
            snapshot_memories.push(mem);
        }

        for dec in decisions.drain(..) {
            let tokens = self.estimate_decision_tokens(&dec, &budget.compression_level);
            if total_tokens + tokens > budget.max_total_tokens {
                break;
            }
            total_tokens += tokens;
            snapshot_decisions.push(dec);
        }

        let summary = match budget.compression_level {
            ContextCompressionLevel::Full => context_analysis.reasoning_summary.clone(),
            ContextCompressionLevel::Compressed => {
                format!("Compressed: {}", context_analysis.reasoning_summary)
            }
            ContextCompressionLevel::ExecutiveSummary => "Executive Summary".to_string(),
        };

        total_tokens += self.estimate_tokens(&summary);

        ReasoningContext {
            memories: snapshot_memories,
            decisions: snapshot_decisions,
            contradictions: context_analysis.contradictions,
            dependencies: context_analysis.dependency_chain,
            timeline,
            confidence: context_analysis.confidence,
            summary,
            estimated_tokens: total_tokens,
        }
    }

    /// Exact token estimator requested: estimated_tokens = (text_length / 4)
    fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }

    fn estimate_memory_tokens(&self, memory: &Memory, level: &ContextCompressionLevel) -> usize {
        match level {
            ContextCompressionLevel::Full => self.estimate_tokens(&memory.content.to_string()),
            ContextCompressionLevel::Compressed => {
                self.estimate_tokens(&memory.content.to_string()) / 2
            } // Assume compressed is half
            ContextCompressionLevel::ExecutiveSummary => self.estimate_tokens(&memory.title),
        }
    }

    fn estimate_decision_tokens(
        &self,
        decision: &Decision,
        level: &ContextCompressionLevel,
    ) -> usize {
        match level {
            ContextCompressionLevel::Full => self.estimate_tokens(&decision.decision_text),
            ContextCompressionLevel::Compressed => self.estimate_tokens(&decision.reason),
            ContextCompressionLevel::ExecutiveSummary => self.estimate_tokens(&decision.title),
        }
    }
}
