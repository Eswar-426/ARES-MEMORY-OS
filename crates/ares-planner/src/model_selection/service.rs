use crate::capability_cache::models::CachedModel;
use ares_core::AresError;

pub struct ModelSelectionService;

impl ModelSelectionService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Assigns capabilities like Reasoning Model, Coding Model, Fast Model, Cheap Model to outputs.
    pub fn select_best_model(&self, task_type: &str) -> Result<CachedModel, AresError> {
        // Placeholder logic: in the future, it balances cost vs capability using knowledge graph

        let selected = if task_type == "coding" {
            CachedModel {
                model_id: "claude-3-5-sonnet".into(),
                max_tokens: 8192,
                cost_per_1k_tokens: 0.015,
                cached_at: chrono::Utc::now(),
            }
        } else {
            CachedModel {
                model_id: "gemini-1.5-flash".into(),
                max_tokens: 4096,
                cost_per_1k_tokens: 0.001,
                cached_at: chrono::Utc::now(),
            }
        };

        Ok(selected)
    }
}
