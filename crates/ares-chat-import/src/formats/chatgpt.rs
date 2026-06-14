use crate::formats::ChatParser;
use crate::types::{ChatConversation, ChatMessage, ConversationRole};
use serde_json::Value;

pub struct ChatGptParser;

impl ChatParser for ChatGptParser {
    fn parse(&self, raw_content: &str) -> Result<ChatConversation, anyhow::Error> {
        let v: Value = serde_json::from_str(raw_content)?;
        let mut messages = Vec::new();

        // This is a simplified version of ChatGPT's conversations.json format.
        // A real implementation would traverse the message mapping.
        if let Some(arr) = v.as_array() {
            for item in arr {
                if let Some(msg_map) = item.get("mapping").and_then(|m| m.as_object()) {
                    for (_id, node) in msg_map {
                        if let Some(msg) = node.get("message") {
                            if msg.is_null() {
                                continue;
                            }

                            let author_role = msg
                                .get("author")
                                .and_then(|a| a.get("role"))
                                .and_then(|r| r.as_str())
                                .unwrap_or("user");

                            let role = match author_role {
                                "assistant" => ConversationRole::Assistant,
                                "system" => ConversationRole::System,
                                _ => ConversationRole::User,
                            };

                            let content = msg
                                .get("content")
                                .and_then(|c| c.get("parts"))
                                .and_then(|p| p.as_array())
                                .and_then(|p| p.first())
                                .and_then(|f| f.as_str())
                                .unwrap_or("");

                            if !content.is_empty() {
                                messages.push(ChatMessage {
                                    role,
                                    content: content.to_string(),
                                    timestamp: msg
                                        .get("create_time")
                                        .and_then(|t| t.as_f64())
                                        .map(|t| t as i64),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(ChatConversation {
            title: Some("ChatGPT Import".to_string()),
            messages,
        })
    }
}
