//! ExtractorProvider trait and MockExtractorProvider.
//!
//! The trait defines the interface for LLM-powered knowledge extraction.
//! MockExtractorProvider provides deterministic results for testing.

use crate::git::CommitInfo;
use ares_core::{KnowledgeCandidate, KnowledgeType};
use async_trait::async_trait;

/// Trait for LLM providers that can extract knowledge from commits.
/// Implementations can target Gemini, Claude, OpenAI, or local models.
#[async_trait]
pub trait ExtractorProvider: Send + Sync {
    /// Analyze a commit and return knowledge candidates.
    async fn extract(&self, commit: &CommitInfo) -> Result<Vec<KnowledgeCandidate>, String>;
}

/// Mock provider for deterministic testing.
/// Analyzes commit messages using keyword heuristics to simulate LLM extraction.
pub struct MockExtractorProvider;

#[async_trait]
impl ExtractorProvider for MockExtractorProvider {
    async fn extract(&self, commit: &CommitInfo) -> Result<Vec<KnowledgeCandidate>, String> {
        let now = ares_core::types::event::now_micros();
        let mut candidates = Vec::new();
        let msg_lower = commit.message.to_lowercase();

        // Simulate decision extraction from keywords
        if msg_lower.contains("decision")
            || msg_lower.contains("chose")
            || msg_lower.contains("selected")
            || msg_lower.contains("rejected")
            || msg_lower.contains("switched")
            || msg_lower.contains("replaced")
        {
            candidates.push(KnowledgeCandidate {
                id: uuid::Uuid::now_v7().to_string(),
                knowledge_type: KnowledgeType::Decision,
                confidence: 0.92,
                reasoning: format!(
                    "Commit message contains decision-related keywords. Message: '{}'",
                    commit.message.lines().next().unwrap_or("")
                ),
                content: format!(
                    "Technical decision made in commit {}: {}",
                    &commit.hash[..8],
                    commit.message.lines().next().unwrap_or("")
                ),
                title: extract_title(&commit.message, "Decision"),
                source_commit: commit.hash.clone(),
                affected_files: commit.files_changed.clone(),
                persisted: false,
                extracted_at: now,
            });
        }

        // Simulate bug extraction
        if msg_lower.contains("fix")
            || msg_lower.contains("bug")
            || msg_lower.contains("issue")
            || msg_lower.contains("error")
            || msg_lower.contains("crash")
            || msg_lower.contains("patch")
        {
            candidates.push(KnowledgeCandidate {
                id: uuid::Uuid::now_v7().to_string(),
                knowledge_type: KnowledgeType::Bug,
                confidence: 0.88,
                reasoning: format!(
                    "Commit message contains bug-fix related keywords. Message: '{}'",
                    commit.message.lines().next().unwrap_or("")
                ),
                content: format!(
                    "Bug fix in commit {}: {}",
                    &commit.hash[..8],
                    commit.message.lines().next().unwrap_or("")
                ),
                title: extract_title(&commit.message, "Bug Fix"),
                source_commit: commit.hash.clone(),
                affected_files: commit.files_changed.clone(),
                persisted: false,
                extracted_at: now,
            });
        }

        // Simulate architecture extraction
        if msg_lower.contains("refactor")
            || msg_lower.contains("architect")
            || msg_lower.contains("restructur")
            || msg_lower.contains("modular")
            || msg_lower.contains("split")
            || msg_lower.contains("extract")
            || msg_lower.contains("new crate")
        {
            candidates.push(KnowledgeCandidate {
                id: uuid::Uuid::now_v7().to_string(),
                knowledge_type: KnowledgeType::Architecture,
                confidence: 0.85,
                reasoning: format!(
                    "Commit message contains architecture-change keywords. Message: '{}'",
                    commit.message.lines().next().unwrap_or("")
                ),
                content: format!(
                    "Architecture change in commit {}: {}",
                    &commit.hash[..8],
                    commit.message.lines().next().unwrap_or("")
                ),
                title: extract_title(&commit.message, "Architecture Change"),
                source_commit: commit.hash.clone(),
                affected_files: commit.files_changed.clone(),
                persisted: false,
                extracted_at: now,
            });
        }

        // Simulate experiment extraction
        if msg_lower.contains("experiment")
            || msg_lower.contains("prototype")
            || msg_lower.contains("spike")
            || msg_lower.contains("poc")
            || msg_lower.contains("try")
            || msg_lower.contains("attempt")
        {
            candidates.push(KnowledgeCandidate {
                id: uuid::Uuid::now_v7().to_string(),
                knowledge_type: KnowledgeType::Experiment,
                confidence: 0.82,
                reasoning: format!(
                    "Commit message contains experimentation keywords. Message: '{}'",
                    commit.message.lines().next().unwrap_or("")
                ),
                content: format!(
                    "Experiment in commit {}: {}",
                    &commit.hash[..8],
                    commit.message.lines().next().unwrap_or("")
                ),
                title: extract_title(&commit.message, "Experiment"),
                source_commit: commit.hash.clone(),
                affected_files: commit.files_changed.clone(),
                persisted: false,
                extracted_at: now,
            });
        }

        // If nothing matched, still produce a low-confidence general note
        if candidates.is_empty() {
            candidates.push(KnowledgeCandidate {
                id: uuid::Uuid::now_v7().to_string(),
                knowledge_type: KnowledgeType::Architecture,
                confidence: 0.40,
                reasoning: "No strong signal detected in commit message. Low confidence general extraction.".into(),
                content: format!(
                    "Code change in commit {}: {}",
                    &commit.hash[..8],
                    commit.message.lines().next().unwrap_or("")
                ),
                title: extract_title(&commit.message, "Code Change"),
                source_commit: commit.hash.clone(),
                affected_files: commit.files_changed.clone(),
                persisted: false,
                extracted_at: now,
            });
        }

        Ok(candidates)
    }
}

/// Extract a short title from a commit message.
fn extract_title(message: &str, fallback_prefix: &str) -> String {
    let first_line = message.lines().next().unwrap_or("").trim();
    if first_line.len() > 80 {
        format!("{}: {}", fallback_prefix, &first_line[..77])
    } else if first_line.is_empty() {
        format!("{}: (no message)", fallback_prefix)
    } else {
        first_line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(message: &str) -> CommitInfo {
        CommitInfo {
            hash: "abc12345def67890abc12345def67890abc12345".into(),
            message: message.into(),
            author: "Test <test@test.com>".into(),
            diff: "".into(),
            files_changed: vec!["src/auth.rs".into()],
        }
    }

    #[tokio::test]
    async fn test_mock_extracts_decision() {
        let provider = MockExtractorProvider;
        let commit = make_commit("feat: selected OAuth2 over Keycloak for auth");
        let candidates = provider.extract(&commit).await.unwrap();
        assert!(candidates
            .iter()
            .any(|c| c.knowledge_type == KnowledgeType::Decision));
    }

    #[tokio::test]
    async fn test_mock_extracts_bug() {
        let provider = MockExtractorProvider;
        let commit = make_commit("fix: resolve token refresh crash on expired sessions");
        let candidates = provider.extract(&commit).await.unwrap();
        assert!(candidates
            .iter()
            .any(|c| c.knowledge_type == KnowledgeType::Bug));
    }

    #[tokio::test]
    async fn test_mock_extracts_architecture() {
        let provider = MockExtractorProvider;
        let commit = make_commit("refactor: extract auth module into separate crate");
        let candidates = provider.extract(&commit).await.unwrap();
        assert!(candidates
            .iter()
            .any(|c| c.knowledge_type == KnowledgeType::Architecture));
    }

    #[tokio::test]
    async fn test_mock_extracts_experiment() {
        let provider = MockExtractorProvider;
        let commit = make_commit("experiment: prototype WebSocket-based real-time sync");
        let candidates = provider.extract(&commit).await.unwrap();
        assert!(candidates
            .iter()
            .any(|c| c.knowledge_type == KnowledgeType::Experiment));
    }

    #[tokio::test]
    async fn test_mock_low_confidence_fallback() {
        let provider = MockExtractorProvider;
        let commit = make_commit("chore: update dependencies");
        let candidates = provider.extract(&commit).await.unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].confidence < 0.80);
    }
}
