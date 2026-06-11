use crate::analysis::prompt::PromptAnalyzer;
use crate::context::manager::ContextManager;
use crate::models::capability::TaskType;
use crate::selection::selector::ModelSelector;
use crate::session::service::SessionService;
use crate::tools::service::ToolIntelligenceService;

pub struct IntelligenceCoordinator {
    pub prompt_analyzer: PromptAnalyzer,
    pub tool_service: ToolIntelligenceService,
    pub context_manager: ContextManager,
    pub model_selector: ModelSelector,
    pub session_service: SessionService,
}

impl Default for IntelligenceCoordinator {
    fn default() -> Self {
        Self::new(
            PromptAnalyzer,
            ToolIntelligenceService::default(),
            ContextManager::default(),
            ModelSelector,
            SessionService::default(),
        )
    }
}

impl IntelligenceCoordinator {
    #[allow(dead_code)]
    pub fn new(
        prompt_analyzer: PromptAnalyzer,
        tool_service: ToolIntelligenceService,
        context_manager: ContextManager,
        model_selector: ModelSelector,
        session_service: SessionService,
    ) -> Self {
        Self {
            prompt_analyzer,
            tool_service,
            context_manager,
            model_selector,
            session_service,
        }
    }

    #[allow(dead_code)]
    pub fn process_task(&self, prompt: &str) -> anyhow::Result<String> {
        let task_type = self
            .prompt_analyzer
            .analyze_task(prompt)
            .unwrap_or(TaskType::Reasoning);
        let _caps = self
            .prompt_analyzer
            .extract_capabilities(prompt)
            .unwrap_or_default();
        let _tools = self.tool_service.resolve_tools_for_task(prompt)?;

        // This coordinates everything together into the main entry point
        // In reality, this would call self.model_selector.select_best_model(...)
        // but we are stubbing it out for now.
        Ok(format!("Task processed as {:?}", task_type))
    }
}
