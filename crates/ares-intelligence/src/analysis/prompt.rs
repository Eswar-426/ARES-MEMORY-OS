use crate::models::capability::{ModelCapability, TaskType};

pub struct PromptAnalyzer;

impl PromptAnalyzer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl Default for PromptAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptAnalyzer {
    #[allow(dead_code)]
    pub fn analyze_task(&self, prompt: &str) -> anyhow::Result<TaskType> {
        if prompt.trim().is_empty() {
            anyhow::bail!("Prompt cannot be empty");
        }
        if prompt.len() > 100_000 {
            anyhow::bail!("Prompt exceeds maximum length");
        }

        let text = prompt.to_lowercase();
        if text.contains("fn ")
            || text.contains("struct ")
            || text.contains("impl ")
            || text.contains("write code")
        {
            return Ok(TaskType::Coding);
        }
        if text.contains("analyze")
            || text.contains("explain")
            || text.contains("why")
            || text.contains("how does")
        {
            return Ok(TaskType::Reasoning);
        }
        if text.contains("summarize") || text.contains("tldr") || text.contains("briefly") {
            return Ok(TaskType::Summarization);
        }
        if text.contains("extract") || text.contains("parse json") {
            return Ok(TaskType::DataExtraction);
        }
        Ok(TaskType::Reasoning)
    }

    #[allow(dead_code)]
    pub fn extract_capabilities(&self, prompt: &str) -> anyhow::Result<Vec<ModelCapability>> {
        if prompt.trim().is_empty() {
            anyhow::bail!("Prompt cannot be empty");
        }

        let mut caps = Vec::new();
        let text = prompt.to_lowercase();

        if text.contains("search")
            || text.contains("find out")
            || text.contains("google")
            || text.contains("look up")
        {
            caps.push(ModelCapability::ToolUse);
        }
        if text.contains("code") || text.contains("function") || text.contains("script") {
            caps.push(ModelCapability::Coding);
        }
        if text.contains("image")
            || text.contains("picture")
            || text.contains("draw")
            || text.contains("photo")
        {
            caps.push(ModelCapability::Vision);
        }

        Ok(caps)
    }
}
