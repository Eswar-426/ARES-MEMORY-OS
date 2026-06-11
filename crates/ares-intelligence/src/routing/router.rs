use crate::models::model::Model;

pub struct TaskRouter;

impl Default for TaskRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskRouter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn route_task(&self, model: &Model) -> anyhow::Result<String> {
        // Simulates picking the endpoint/provider for a model
        if model.provider_id.is_empty() {
            anyhow::bail!("Model has no provider specified");
        }

        let endpoint = format!(
            "https://api.{}.mock/v1/execute",
            model.provider_id.to_lowercase()
        );
        Ok(endpoint)
    }
}
