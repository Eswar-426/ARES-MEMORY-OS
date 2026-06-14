use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// The name of the model provider (e.g., "Gemini", "OpenRouter")
    fn name(&self) -> &str;

    /// Sends a prompt to the model and returns the response string.
    /// The context represents the PortableContext, and the prompt is the question.
    async fn ask(&self, context: &str, prompt: &str) -> Result<String>;
}
