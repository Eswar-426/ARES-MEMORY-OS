use crate::events::envelope::EventEnvelope;
use ares_core::AresError;
use async_trait::async_trait;

#[async_trait]
pub trait Projection: Send + Sync {
    /// Applies an event to build read-model state.
    async fn apply(&self, event: &EventEnvelope) -> Result<(), AresError>;
}

pub struct ProjectionEngine {
    projections: Vec<Box<dyn Projection>>,
}

impl Default for ProjectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectionEngine {
    pub fn new() -> Self {
        Self {
            projections: Vec::new(),
        }
    }

    pub fn register(&mut self, projection: Box<dyn Projection>) {
        self.projections.push(projection);
    }

    pub async fn process_event(&self, event: &EventEnvelope) -> Result<(), AresError> {
        for proj in &self.projections {
            proj.apply(event).await?;
        }
        Ok(())
    }
}
