use crate::formats::ChatParser;
use crate::types::{ChatConversation, ChatMessage, ConversationRole};
use serde_json::Value;

pub struct ClaudeParser;

impl ChatParser for ClaudeParser {
    fn parse(&self, raw_content: &str) -> Result<ChatConversation, anyhow::Error> {
        let v: Value = serde_json::from_str(raw_content)?;
        let mut messages = Vec::new();

        if let Some(arr) = v.as_array() {
            for item in arr {
                if let Some(chat_messages) = item.get("chat_messages").and_then(|m| m.as_array()) {
                    for msg in chat_messages {
                        let role_str = msg.get("sender").and_then(|s| s.as_str()).unwrap_or("user");
                        let role = match role_str {
                            "assistant" | "claude" => ConversationRole::Assistant,
                            _ => ConversationRole::User,
                        };

                        let content = msg.get("text").and_then(|t| t.as_str()).unwrap_or("");

                        if !content.is_empty() {
                            messages.push(ChatMessage {
                                role,
                                content: content.to_string(),
                                timestamp: msg
                                    .get("created_at")
                                    .and_then(|t| t.as_str())
                                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                                    .map(|d| d.timestamp()),
                            });
                        }
                    }
                }
            }
        }

        Ok(ChatConversation {
            title: Some("Claude Import".to_string()),
            messages,
        })
    }
}
