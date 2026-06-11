use crate::models::model::Model;

pub struct FallbackManager;

impl Default for FallbackManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FallbackManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn get_fallback_model(
        &self,
        failed_model: &Model,
        available_models: &[Model],
    ) -> Option<Model> {
        // Simplified fallback: Just pick the first available model that isn't the failed one
        available_models
            .iter()
            .find(|m| m.id != failed_model.id)
            .cloned()
    }
}
