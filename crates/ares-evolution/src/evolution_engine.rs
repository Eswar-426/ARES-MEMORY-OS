use ares_core::types::evolution::EvolutionEvent;
use ares_store::repositories::evolution::EvolutionRepository;
use std::sync::Arc;

pub struct EvolutionEngine {
    repo: Arc<dyn EvolutionRepository>,
}

impl EvolutionEngine {
    pub fn new(repo: Arc<dyn EvolutionRepository>) -> Self {
        Self { repo }
    }

    /// Records an evolution event into the history.
    pub async fn record_event(
        &self,
        project_id: &str,
        event: &EvolutionEvent,
    ) -> Result<(), String> {
        self.repo.record_event(project_id, event).await
    }

    /// Retrieves all evolution events associated with a specific target node.
    pub async fn get_events_for_node(
        &self,
        project_id: &str,
        target_node: &str,
    ) -> Result<Vec<EvolutionEvent>, String> {
        self.repo.get_events_for_node(project_id, target_node).await
    }
}
