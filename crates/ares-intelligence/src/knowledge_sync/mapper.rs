use super::events::KnowledgeSyncEvent;

pub struct KnowledgeMapper;

impl Default for KnowledgeMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeMapper {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn map_to_event(&self, memory: &str, confidence: f64) -> KnowledgeSyncEvent {
        KnowledgeSyncEvent {
            entity_id: uuid::Uuid::now_v7().to_string(),
            payload: memory.to_string(),
            confidence,
        }
    }
}
