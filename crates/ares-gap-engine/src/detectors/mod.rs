use crate::models::Gap;
use ares_core::{AresError, id::ProjectId};
use ares_store::Store;
use async_trait::async_trait;
use std::sync::Arc;

pub mod decisions;
pub mod requirements;
pub mod traceability;

#[async_trait]
pub trait GapDetector: Send + Sync {
    /// Returns a list of gap types this detector is capable of finding
    fn supported_types(&self) -> Vec<crate::models::GapType>;
    
    /// Executes the detection logic for a given project
    async fn detect(&self, project_id: &ProjectId, store: Arc<Store>) -> Result<Vec<Gap>, AresError>;
}
