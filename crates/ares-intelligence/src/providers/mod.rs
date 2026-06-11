pub mod claude;
pub mod error;
pub mod gemini;
pub mod mock;
pub mod openai;
pub mod registry;
pub mod types;

use async_trait::async_trait;
pub use error::ProviderError;
pub use types::{ModelRequest, ModelResponse, ProviderHealthStatus, ProviderMetadata};

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn generate(&self, request: ModelRequest) -> Result<ModelResponse, ProviderError>;
    async fn health_check(&self) -> ProviderHealthStatus;
    fn metadata(&self) -> ProviderMetadata;
}
