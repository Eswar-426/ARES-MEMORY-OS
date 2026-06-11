use crate::models::capability::ModelCapability;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModelCatalogEntry {
    pub id: String,
    pub provider_id: String,
    pub cost_per_input_token: f64,
    pub cost_per_output_token: f64,
    pub max_context: usize,
    pub capabilities: Vec<ModelCapability>,
    pub latency_ms: u64,
    pub quality_score: f32,
}

pub struct ModelCatalog {
    entries: HashMap<String, ModelCatalogEntry>,
}

impl Default for ModelCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelCatalog {
    pub fn new() -> Self {
        let mut entries = HashMap::new();

        // Gemini models
        entries.insert(
            "gemini-2.5-pro".to_string(),
            ModelCatalogEntry {
                id: "gemini-2.5-pro".to_string(),
                provider_id: "gemini".to_string(),
                cost_per_input_token: 0.00000125,
                cost_per_output_token: 0.00000375,
                max_context: 2000000,
                capabilities: vec![
                    ModelCapability::Reasoning,
                    ModelCapability::Coding,
                    ModelCapability::Vision,
                    ModelCapability::LongContext,
                    ModelCapability::ToolUse,
                ],
                latency_ms: 1500,
                quality_score: 9.5,
            },
        );

        entries.insert(
            "gemini-2.5-flash".to_string(),
            ModelCatalogEntry {
                id: "gemini-2.5-flash".to_string(),
                provider_id: "gemini".to_string(),
                cost_per_input_token: 0.00000015,
                cost_per_output_token: 0.00000060,
                max_context: 1000000,
                capabilities: vec![
                    ModelCapability::Reasoning,
                    ModelCapability::Coding,
                    ModelCapability::Vision,
                    ModelCapability::FastResponse,
                    ModelCapability::LowCost,
                    ModelCapability::ToolUse,
                ],
                latency_ms: 500,
                quality_score: 8.8,
            },
        );

        // OpenAI models
        entries.insert(
            "gpt-4o".to_string(),
            ModelCatalogEntry {
                id: "gpt-4o".to_string(),
                provider_id: "openai".to_string(),
                cost_per_input_token: 0.000005,
                cost_per_output_token: 0.000015,
                max_context: 128000,
                capabilities: vec![
                    ModelCapability::Reasoning,
                    ModelCapability::Coding,
                    ModelCapability::Vision,
                    ModelCapability::ToolUse,
                ],
                latency_ms: 1200,
                quality_score: 9.6,
            },
        );

        entries.insert(
            "gpt-4o-mini".to_string(),
            ModelCatalogEntry {
                id: "gpt-4o-mini".to_string(),
                provider_id: "openai".to_string(),
                cost_per_input_token: 0.00000015,
                cost_per_output_token: 0.00000060,
                max_context: 128000,
                capabilities: vec![
                    ModelCapability::Reasoning,
                    ModelCapability::Coding,
                    ModelCapability::Vision,
                    ModelCapability::FastResponse,
                    ModelCapability::LowCost,
                    ModelCapability::ToolUse,
                ],
                latency_ms: 400,
                quality_score: 8.7,
            },
        );

        // Claude models
        entries.insert(
            "claude-3-opus".to_string(),
            ModelCatalogEntry {
                id: "claude-3-opus".to_string(),
                provider_id: "claude".to_string(),
                cost_per_input_token: 0.000015,
                cost_per_output_token: 0.000075,
                max_context: 200000,
                capabilities: vec![
                    ModelCapability::Reasoning,
                    ModelCapability::Coding,
                    ModelCapability::Vision,
                    ModelCapability::LongContext,
                    ModelCapability::ToolUse,
                ],
                latency_ms: 2000,
                quality_score: 9.7,
            },
        );

        entries.insert(
            "claude-3-5-sonnet".to_string(),
            ModelCatalogEntry {
                id: "claude-3-5-sonnet".to_string(),
                provider_id: "claude".to_string(),
                cost_per_input_token: 0.000003,
                cost_per_output_token: 0.000015,
                max_context: 200000,
                capabilities: vec![
                    ModelCapability::Reasoning,
                    ModelCapability::Coding,
                    ModelCapability::Vision,
                    ModelCapability::FastResponse,
                    ModelCapability::ToolUse,
                ],
                latency_ms: 1000,
                quality_score: 9.6,
            },
        );

        // Mock model
        entries.insert(
            "mock-model".to_string(),
            ModelCatalogEntry {
                id: "mock-model".to_string(),
                provider_id: "mock".to_string(),
                cost_per_input_token: 0.0,
                cost_per_output_token: 0.0,
                max_context: 8000,
                capabilities: vec![
                    ModelCapability::Reasoning,
                    ModelCapability::FastResponse,
                    ModelCapability::LowCost,
                ],
                latency_ms: 10,
                quality_score: 5.0,
            },
        );

        Self { entries }
    }

    pub fn get(&self, id: &str) -> Option<&ModelCatalogEntry> {
        self.entries.get(id)
    }

    pub fn list_all(&self) -> Vec<&ModelCatalogEntry> {
        self.entries.values().collect()
    }
}
