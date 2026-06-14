use crate::formats::ChatParser;
use crate::types::{ChatConversation, ChatMessage, ConversationRole};
use serde_json::Value;

pub struct JsonGenericParser;

impl ChatParser for JsonGenericParser {
    fn parse(&self, raw_content: &str) -> Result<ChatConversation, anyhow::Error> {
        let v: Value = serde_json::from_str(raw_content)?;
        let mut messages = Vec::new();

        if let Some(arr) = v.as_array() {
            for item in arr {
                let role_str = item
                    .get("role")
                    .and_then(|s| s.as_str())
                    .unwrap_or("user")
                    .to_lowercase();
                let role = match role_str.as_str() {
                    "assistant" | "system" | "bot" | "ai" => ConversationRole::Assistant,
                    _ => ConversationRole::User,
                };

                let content = item
                    .get("content")
                    .or_else(|| item.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                if !content.is_empty() {
                    messages.push(ChatMessage {
                        role,
                        content: content.to_string(),
                        timestamp: item.get("timestamp").and_then(|t| t.as_i64()),
                    });
                }
            }
        } else if let Some(obj) = v.as_object() {
            if let Some(arr) = obj.get("messages").and_then(|m| m.as_array()) {
                for item in arr {
                    let role_str = item
                        .get("role")
                        .and_then(|s| s.as_str())
                        .unwrap_or("user")
                        .to_lowercase();
                    let role = match role_str.as_str() {
                        "assistant" | "system" | "bot" | "ai" => ConversationRole::Assistant,
                        _ => ConversationRole::User,
                    };

                    let content = item
                        .get("content")
                        .or_else(|| item.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");

                    if !content.is_empty() {
                        messages.push(ChatMessage {
                            role,
                            content: content.to_string(),
                            timestamp: item.get("timestamp").and_then(|t| t.as_i64()),
                        });
                    }
                }
            }
        }

        Ok(ChatConversation {
            title: Some("Generic JSON Import".to_string()),
            messages,
        })
    }
}
