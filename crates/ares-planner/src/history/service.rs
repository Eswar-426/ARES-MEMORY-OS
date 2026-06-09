use ares_core::id::PlanId;
use ares_core::AresError;

pub struct HistoryService;

impl HistoryService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Translates feedback into a Knowledge Graph format.
    pub fn push_to_knowledge_graph(
        &self,
        _plan_id: &PlanId,
        _lessons_learned: &str,
    ) -> Result<(), AresError> {
        // Week 11 Integration point!
        // This takes what `FeedbackService` learned and persists it
        // to `ares-knowledge` so the LLM has it for future Context Retrieval.
        Ok(())
    }
}
