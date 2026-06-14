use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use super::r#trait::ModelProvider;

pub struct GeminiProvider {
    api_key: String,
    client: Client,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: &str) -> Self {
        Self {
            api_key,
            client: Client::new(),
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    fn name(&self) -> &str {
        "Google Gemini"
    }

    async fn ask(&self, context: &str, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let combined_prompt = format!("{}\n\nQuestion: {}", context, prompt);

        let body = json!({
            "contents": [{
                "parts": [{
                    "text": combined_prompt
                }]
            }]
        });

        let res = self.client.post(&url).json(&body).send().await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await?;
            return Err(anyhow!("Gemini API error: {} - {}", status, text));
        }

        let json_res: serde_json::Value = res.json().await?;

        // Parse Gemini response structure
        let text = json_res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(text)
    }
}
