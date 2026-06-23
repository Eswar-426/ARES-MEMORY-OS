//! Types for the Context Generator.

use serde::{Deserialize, Serialize};

/// A portable context payload that can be sent to any AI model.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PortableContext {
    /// The full rendered text, ready to paste into an AI context window.
    pub text: String,
    /// Structured sections for programmatic access.
    pub sections: Vec<ContextSection>,
    /// Estimated token count (4 chars ≈ 1 token).
    pub estimated_tokens: usize,
    /// The project this context was generated for.
    pub project_name: String,
    /// When this context was generated.
    pub generated_at: i64,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct ContextSection {
    pub title: String,
    pub content: String,
    pub priority: SectionPriority,
    pub estimated_tokens: usize,
}

#[derive(
    utoipa::ToSchema, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum SectionPriority {
    /// Must always be included
    Critical = 0,
    /// Include if budget allows
    High = 1,
    /// Include if plenty of budget
    Medium = 2,
    /// Include only if extra space
    Low = 3,
}

/// Output format for the context generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Markdown format (good for Claude, Gemini, ChatGPT)
    #[default]
    Markdown,
    /// Plain text (minimal formatting)
    PlainText,
    /// Structured JSON
    Json,
}
