pub mod chatgpt;
pub mod claude;
pub mod cursor;
pub mod gemini;
pub mod json_generic;
pub mod markdown;

use crate::types::{ChatConversation, ChatFormat};

pub trait ChatParser {
    fn parse(&self, raw_content: &str) -> Result<ChatConversation, anyhow::Error>;
}

pub fn get_parser(format: &ChatFormat) -> Box<dyn ChatParser> {
    match format {
        ChatFormat::ChatGPT => Box::new(chatgpt::ChatGptParser),
        ChatFormat::Claude => Box::new(claude::ClaudeParser),
        ChatFormat::Gemini => Box::new(gemini::GeminiParser),
        ChatFormat::Cursor => Box::new(cursor::CursorParser),
        ChatFormat::Markdown => Box::new(markdown::MarkdownParser),
        ChatFormat::JsonGeneric => Box::new(json_generic::JsonGenericParser),
    }
}
