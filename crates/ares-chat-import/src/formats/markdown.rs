use crate::formats::ChatParser;
use crate::types::{ChatConversation, ChatMessage, ConversationRole};

pub struct MarkdownParser;

impl ChatParser for MarkdownParser {
    fn parse(&self, raw_content: &str) -> Result<ChatConversation, anyhow::Error> {
        let mut messages = Vec::new();
        let mut current_role = ConversationRole::User;
        let mut current_content = String::new();

        for line in raw_content.lines() {
            let lower = line.to_lowercase();
            if lower.starts_with("**user:**")
                || lower.starts_with("user:")
                || lower.starts_with("## user")
            {
                if !current_content.trim().is_empty() {
                    messages.push(ChatMessage {
                        role: current_role,
                        content: current_content.trim().to_string(),
                        timestamp: None,
                    });
                }
                current_role = ConversationRole::User;
                current_content.clear();
            } else if lower.starts_with("**assistant:**")
                || lower.starts_with("assistant:")
                || lower.starts_with("## assistant")
            {
                if !current_content.trim().is_empty() {
                    messages.push(ChatMessage {
                        role: current_role,
                        content: current_content.trim().to_string(),
                        timestamp: None,
                    });
                }
                current_role = ConversationRole::Assistant;
                current_content.clear();
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        if !current_content.trim().is_empty() {
            messages.push(ChatMessage {
                role: current_role,
                content: current_content.trim().to_string(),
                timestamp: None,
            });
        }

        Ok(ChatConversation {
            title: Some("Markdown Import".to_string()),
            messages,
        })
    }
}
