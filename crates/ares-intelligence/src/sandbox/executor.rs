use super::limits::Limits;
use super::quotas::Quotas;
use crate::providers::ModelProvider;
use std::time::Duration;

#[allow(dead_code)]
pub struct SandboxExecutor {
    limits: Limits,
    quotas: Quotas,
}

impl SandboxExecutor {
    pub fn new(limits: Limits, quotas: Quotas) -> Self {
        Self { limits, quotas }
    }

    pub async fn execute(
        &self,
        provider: &dyn ModelProvider,
        prompt: &str,
    ) -> anyhow::Result<String> {
        let estimated_tokens = prompt.len() / 4;
        if estimated_tokens > self.limits.max_tokens_per_minute as usize {
            anyhow::bail!("Token limit exceeded");
        }

        let request = crate::providers::types::ModelRequest {
            prompt: prompt.to_string(),
            max_tokens: None,
            temperature: None,
            stream: false,
        };
        let fut = provider.generate(request);
        let timeout_duration = Duration::from_millis(100);

        match tokio::time::timeout(timeout_duration, fut).await {
            Ok(Ok(result)) => Ok(result.content),
            Ok(Err(e)) => Err(anyhow::anyhow!("Provider error: {:?}", e)),
            Err(_) => anyhow::bail!("Execution timeout"),
        }
    }
}
