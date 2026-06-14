use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatFormat {
    ChatGPT,
    Claude,
    Gemini,
    Cursor,
    Markdown,
    JsonGeneric,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ConversationRole,
    pub content: String,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConversation {
    pub title: Option<String>,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityType {
    Decision,
    Feature,
    Bug,
}

#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    pub entity_type: EntityType,
    pub content: String,
    pub context: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct ImportContext {
    pub project_id: String,
    pub format: ChatFormat,
}
