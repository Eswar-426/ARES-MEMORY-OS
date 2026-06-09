use std::sync::Arc;
use tokio::time::{sleep, Duration};
use super::processor::EventProcessor;

pub struct KnowledgeIngestionWorker {
    _processor: Arc<EventProcessor>,
}

impl KnowledgeIngestionWorker {
    pub fn new(processor: Arc<EventProcessor>) -> Self {
        Self { _processor: processor }
    }

    pub async fn run(&self) {
        loop {
            // Poll for pending events from knowledge_events table
            // Process them using EventProcessor
            sleep(Duration::from_secs(5)).await;
        }
    }
}
