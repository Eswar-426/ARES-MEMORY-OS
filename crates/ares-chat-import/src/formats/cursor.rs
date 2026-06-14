use crate::formats::ChatParser;
use crate::types::{ChatConversation, ChatMessage, ConversationRole};
use serde_json::Value;

pub struct CursorParser;

impl ChatParser for CursorParser {
    fn parse(&self, raw_content: &str) -> Result<ChatConversation, anyhow::Error> {
        let v: Value = serde_json::from_str(raw_content)?;
        let mut messages = Vec::new();

        if let Some(arr) = v.as_array() {
            for item in arr {
                let role_type = item
                    .get("type")
                    .and_then(|s| s.as_number())
                    .and_then(|n| n.as_i64())
                    .unwrap_or(1);
                let role = match role_type {
                    2 => ConversationRole::Assistant,
                    _ => ConversationRole::User,
                };

                let content = item.get("text").and_then(|t| t.as_str()).unwrap_or("");

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
            title: Some("Cursor Import".to_string()),
            messages,
        })
    }
}
