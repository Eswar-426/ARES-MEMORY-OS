use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

pub struct RetryEngine {
    pub max_retries: u32,
    pub base_backoff_ms: u64,
}

impl Default for RetryEngine {
    fn default() -> Self {
        Self::new(3, 100)
    }
}

impl RetryEngine {
    pub fn new(max_retries: u32, base_backoff_ms: u64) -> Self {
        Self {
            max_retries,
            base_backoff_ms,
        }
    }

    pub async fn execute_with_retry<F, Fut, T>(&self, mut action: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        loop {
            match action().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    attempt += 1;
                    if attempt > self.max_retries {
                        return Err(err);
                    }

                    // Simple exponential backoff
                    let backoff = self.base_backoff_ms * (2_u64.pow(attempt - 1));
                    sleep(Duration::from_millis(backoff)).await;
                }
            }
        }
    }
}
