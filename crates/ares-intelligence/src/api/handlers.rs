// In a real implementation this would hold axum/actix handlers
pub struct IntelligenceApiHandler;

impl Default for IntelligenceApiHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl IntelligenceApiHandler {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn handle_process_task_request(&self, _payload: &str) -> anyhow::Result<String> {
        Ok("Accepted".to_string())
    }
}
