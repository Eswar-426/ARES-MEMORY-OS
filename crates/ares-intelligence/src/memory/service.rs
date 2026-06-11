use super::compression::ContextCompressor;
use super::retrieval::MemoryRetrievalEngine;
use async_trait::async_trait;

#[async_trait]
pub trait MemoryIntelligenceManager: Send + Sync {
    async fn prepare_context(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<String>;
}

#[allow(dead_code)]
pub struct MemoryIntelligenceService {
    retrieval: MemoryRetrievalEngine,
    compressor: ContextCompressor,
}

impl MemoryIntelligenceService {
    #[allow(dead_code)]
    pub fn new(retrieval: MemoryRetrievalEngine, compressor: ContextCompressor) -> Self {
        Self {
            retrieval,
            compressor,
        }
    }
}

#[async_trait]
impl MemoryIntelligenceManager for MemoryIntelligenceService {
    async fn prepare_context(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<String> {
        let memories = self.retrieval.retrieve_relevant_memories(prompt, 5).await?;
        let combined = memories.join("\n");
        self.compressor.compress(&combined, max_tokens)
    }
}
