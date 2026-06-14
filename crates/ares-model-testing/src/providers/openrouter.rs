use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use super::r#trait::ModelProvider;

pub struct OpenRouterProvider {
    api_key: String,
    client: Client,
    model: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String, model: &str) -> Self {
        Self {
            api_key,
            client: Client::new(),
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl ModelProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        &self.model
    }

    async fn ask(&self, context: &str, prompt: &str) -> Result<String> {
        let url = "https://openrouter.ai/api/v1/chat/completions";

        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a software engineer reviewing a project's context."
                },
                {
                    "role": "user",
                    "content": format!("{}\n\nQuestion: {}", context, prompt)
                }
            ]
        });

        let res = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://ares-memory-os.local")
            .header("X-Title", "ARES Memory OS Tests")
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await?;
            return Err(anyhow!("OpenRouter API error: {} - {}", status, text));
        }

        let json_res: serde_json::Value = res.json().await?;

        let text = json_res["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(text)
    }
}
