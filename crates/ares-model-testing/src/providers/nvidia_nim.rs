use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use super::r#trait::ModelProvider;

pub struct NvidiaNimProvider {
    api_key: String,
    client: Client,
    model: String,
}

impl NvidiaNimProvider {
    pub fn new(api_key: String, model: &str) -> Self {
        Self {
            api_key,
            client: Client::new(),
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl ModelProvider for NvidiaNimProvider {
    fn name(&self) -> &str {
        "NVIDIA NIM (Llama-3 70B)"
    }

    async fn ask(&self, context: &str, prompt: &str) -> Result<String> {
        let url = "https://integrate.api.nvidia.com/v1/chat/completions";

        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": format!("{}\n\nQuestion: {}", context, prompt)
                }
            ],
            "max_tokens": 1024,
            "temperature": 0.2
        });

        let res = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await?;
            return Err(anyhow!("NVIDIA NIM API error: {} - {}", status, text));
        }

        let json_res: serde_json::Value = res.json().await?;

        let text = json_res["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(text)
    }
}
