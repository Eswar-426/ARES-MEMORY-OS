use crate::formats::ChatParser;
use crate::types::{ChatConversation, ChatMessage, ConversationRole};
use serde_json::Value;

pub struct GeminiParser;

impl ChatParser for GeminiParser {
    fn parse(&self, raw_content: &str) -> Result<ChatConversation, anyhow::Error> {
        let v: Value = serde_json::from_str(raw_content)?;
        let mut messages = Vec::new();

        if let Some(arr) = v.as_array() {
            for item in arr {
                let role_str = item.get("role").and_then(|s| s.as_str()).unwrap_or("user");
                let role = match role_str {
                    "model" => ConversationRole::Assistant,
                    _ => ConversationRole::User,
                };

                let content = item
                    .get("parts")
                    .and_then(|p| p.as_array())
                    .and_then(|p| p.first())
                    .and_then(|f| f.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                if !content.is_empty() {
                    messages.push(ChatMessage {
                        role,
                        content: content.to_string(),
                        timestamp: None,
                    });
                }
            }
        }

        Ok(ChatConversation {
            title: Some("Gemini Import".to_string()),
            messages,
        })
    }
}
