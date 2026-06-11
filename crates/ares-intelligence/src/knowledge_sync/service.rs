use super::events::KnowledgeSyncEvent;
use super::mapper::KnowledgeMapper;

pub struct KnowledgeSyncService {
    mapper: KnowledgeMapper,
}

impl Default for KnowledgeSyncService {
    fn default() -> Self {
        Self::new(KnowledgeMapper)
    }
}

impl KnowledgeSyncService {
    #[allow(dead_code)]
    pub fn new(mapper: KnowledgeMapper) -> Self {
        Self { mapper }
    }

    #[allow(dead_code)]
    pub fn sync_learned_memory(
        &self,
        memory: &str,
        confidence: f64,
    ) -> anyhow::Result<KnowledgeSyncEvent> {
        let event = self.mapper.map_to_event(memory, confidence);
        // Dispatch to Knowledge Graph system
        Ok(event)
    }
}
