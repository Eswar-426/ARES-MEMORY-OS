use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelCapability {
    Reasoning,
    Coding,
    Vision,
    Embedding,
    FastResponse,
    LowCost,
    LongContext,
    ToolUse,
    Planning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    Reasoning,
    Coding,
    Summarization,
    Research,
    Planning,
    Memory,
    Vision,
    DataExtraction,
}
