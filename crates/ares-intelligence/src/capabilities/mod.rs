use crate::catalog::ModelCatalogEntry;
use crate::models::capability::{ModelCapability, TaskType};

pub struct CapabilityValidator;

impl CapabilityValidator {
    /// Validates if the given model meets all requirements for a specific task.
    pub fn validate_model_for_task(
        model: &ModelCatalogEntry,
        required_capabilities: &[ModelCapability],
        required_context: usize,
    ) -> bool {
        Self::supports_capabilities(model, required_capabilities)
            && Self::supports_context_window(model, required_context)
    }

    /// Checks if the model has all the requested capabilities.
    pub fn supports_capabilities(
        model: &ModelCatalogEntry,
        required_capabilities: &[ModelCapability],
    ) -> bool {
        required_capabilities
            .iter()
            .all(|cap| model.capabilities.contains(cap))
    }

    /// Checks if the model's context window is sufficient.
    pub fn supports_context_window(model: &ModelCatalogEntry, required_context: usize) -> bool {
        model.max_context >= required_context
    }

    /// Checks if the model explicitly supports tool use.
    pub fn supports_tools(model: &ModelCatalogEntry) -> bool {
        model.capabilities.contains(&ModelCapability::ToolUse)
    }

    /// Helper to map a high-level TaskType to minimum ModelCapability requirements.
    pub fn capabilities_for_task_type(task_type: TaskType) -> Vec<ModelCapability> {
        match task_type {
            TaskType::Reasoning => vec![ModelCapability::Reasoning],
            TaskType::Coding => vec![ModelCapability::Coding, ModelCapability::Reasoning],
            TaskType::Vision => vec![ModelCapability::Vision],
            TaskType::Summarization => vec![ModelCapability::Reasoning],
            TaskType::Research => vec![ModelCapability::Reasoning, ModelCapability::LongContext],
            TaskType::Planning => vec![ModelCapability::Planning, ModelCapability::Reasoning],
            TaskType::Memory => vec![],
            TaskType::DataExtraction => vec![ModelCapability::Reasoning, ModelCapability::ToolUse],
        }
    }
}
