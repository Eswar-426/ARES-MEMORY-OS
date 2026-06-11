use super::capability::ToolCapability;
use crate::analysis::prompt::PromptAnalyzer;

pub struct ToolSelector {
    prompt_analyzer: PromptAnalyzer,
}

impl Default for ToolSelector {
    fn default() -> Self {
        Self::new(PromptAnalyzer)
    }
}

impl ToolSelector {
    #[allow(dead_code)]
    pub fn new(prompt_analyzer: PromptAnalyzer) -> Self {
        Self { prompt_analyzer }
    }

    #[allow(dead_code)]
    pub fn select_tools(&self, prompt: &str) -> anyhow::Result<Vec<ToolCapability>> {
        let mut caps = Vec::new();
        let text = prompt.to_lowercase();

        if text.contains("search") || text.contains("find out") {
            caps.push(ToolCapability::Search);
        }
        if text.contains("run code") || text.contains("execute script") {
            caps.push(ToolCapability::CodeExecution);
        }
        if text.contains("read file") || text.contains("write to file") {
            caps.push(ToolCapability::Filesystem);
        }
        if text.contains("browse") || text.contains("web") {
            caps.push(ToolCapability::Browser);
        }
        if text.contains("knowledge") || text.contains("graph") {
            caps.push(ToolCapability::KnowledgeGraph);
        }

        // Also use prompt_analyzer for cross checks
        if let Ok(model_caps) = self.prompt_analyzer.extract_capabilities(prompt) {
            use crate::models::capability::ModelCapability;
            if model_caps.contains(&ModelCapability::ToolUse)
                && !caps.contains(&ToolCapability::Search)
            {
                caps.push(ToolCapability::Search);
            }
        }

        Ok(caps)
    }
}
