use anyhow::{anyhow, Result};
use std::env;

/// Trait abstracting secret management, allowing future extension to Vault, GCP Secrets, etc.
#[async_trait::async_trait]
pub trait SecretManager: Send + Sync {
    async fn get_secret(&self, key: &str) -> Result<String>;
    async fn refresh_secret(&self, key: &str) -> Result<String>;
}

/// Simple environment variable-backed secret manager.
pub struct EnvSecretManager;

impl Default for EnvSecretManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvSecretManager {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SecretManager for EnvSecretManager {
    async fn get_secret(&self, key: &str) -> Result<String> {
        env::var(key).map_err(|_| anyhow!("Secret not found in environment: {}", key))
    }

    async fn refresh_secret(&self, key: &str) -> Result<String> {
        // For environment variables, refresh just reads it again
        self.get_secret(key).await
    }
}
