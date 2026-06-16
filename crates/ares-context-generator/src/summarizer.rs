//! MemorySummarizer — condenses lists of decisions, features, bugs into concise summaries.

use ares_project_memory::types::*;

pub struct MemorySummarizer;

impl MemorySummarizer {
    /// Summarize decisions into bullet points.
    pub fn summarize_decisions(decisions: &[ares_decision_intelligence::DecisionSummary]) -> Vec<String> {
        if decisions.is_empty() {
            return vec!["No decisions recorded yet.".into()];
        }

        decisions
            .iter()
            .map(|d| {
                format!(
                    "{} — {}",
                    crate::templates::status_badge(d.approval_status.as_str()),
                    d.title
                )
            })
            .collect()
    }

    /// Summarize features with status counts.
    pub fn summarize_features(features: &[FeatureSummary]) -> (String, Vec<String>) {
        if features.is_empty() {
            return ("No features tracked.".into(), vec![]);
        }

        let total = features.len();
        let active = features.iter().filter(|f| f.status == "active").count();
        let completed = features
            .iter()
            .filter(|f| f.status == "archived" || f.status == "deprecated")
            .count();

        let summary = format!(
            "{} features ({} active, {} completed)",
            total, active, completed
        );

        let items: Vec<String> = features
            .iter()
            .map(|f| format!("{} {}", crate::templates::status_badge(&f.status), f.title,))
            .collect();

        (summary, items)
    }

    /// Summarize bugs with severity breakdown.
    pub fn summarize_bugs(bugs: &[BugSummary]) -> (String, Vec<String>) {
        if bugs.is_empty() {
            return ("No bugs recorded.".into(), vec![]);
        }

        let total = bugs.len();
        let critical = bugs.iter().filter(|b| b.severity == "critical").count();
        let high = bugs.iter().filter(|b| b.severity == "high").count();

        let summary = format!(
            "{} bugs ({} critical, {} high priority)",
            total, critical, high
        );

        let items: Vec<String> = bugs
            .iter()
            .map(|b| format!("[{}] {} ({})", b.severity.to_uppercase(), b.title, b.status,))
            .collect();

        (summary, items)
    }

    /// Summarize recent changes into a timeline.
    pub fn summarize_changes(changes: &[ChangeRecord]) -> Vec<String> {
        if changes.is_empty() {
            return vec!["No recent changes recorded.".into()];
        }

        changes
            .iter()
            .take(20)
            .map(|c| {
                let type_str = match c.change_type {
                    ChangeType::MemoryCreated => "📝 Created",
                    ChangeType::MemoryUpdated => "✏️ Updated",
                    ChangeType::DecisionMade => "⚖️ Decision",
                    ChangeType::ScanCompleted => "🔍 Scan",
                    ChangeType::FeatureAdded => "✨ Feature",
                    ChangeType::BugFixed => "🐛 Fix",
                };
                format!("{}: {}", type_str, c.description)
            })
            .collect()
    }

    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_empty_decisions() {
        let result = MemorySummarizer::summarize_decisions(&[]);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("No decisions"));
    }

    #[test]
    fn summarize_decisions_with_items() {
        let decisions = vec![ares_decision_intelligence::DecisionSummary {
            id: "d1".into(),
            title: "Use Axum for HTTP".into(),
            approval_status: ares_decision_intelligence::DecisionStatus::Approved,
        }];
        let result = MemorySummarizer::summarize_decisions(&decisions);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("Axum"));
    }

    #[test]
    fn summarize_empty_features() {
        let (summary, items) = MemorySummarizer::summarize_features(&[]);
        assert!(summary.contains("No features"));
        assert!(items.is_empty());
    }

    #[test]
    fn summarize_changes_with_items() {
        let changes = vec![ChangeRecord {
            change_type: ChangeType::DecisionMade,
            description: "Chose SQLite".into(),
            files_affected: vec![],
            timestamp: 0,
        }];
        let result = MemorySummarizer::summarize_changes(&changes);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("Decision"));
    }
}
