//! ContextCompressor — fits context into AI model token budgets.

use crate::types::{ContextSection, PortableContext, SectionPriority};
use tracing::debug;

pub struct ContextCompressor;

impl ContextCompressor {
    /// Estimate token count for a string (heuristic: 4 chars ≈ 1 token).
    pub fn estimate_tokens(text: &str) -> usize {
        text.len().div_ceil(4)
    }

    /// Compress a PortableContext to fit within a token budget.
    /// Removes sections by priority (Low first, then Medium, then High).
    /// Critical sections are never removed.
    pub fn compress(context: PortableContext, max_tokens: usize) -> PortableContext {
        let current_tokens = Self::estimate_tokens(&context.text);

        if current_tokens <= max_tokens {
            debug!(
                current = current_tokens,
                budget = max_tokens,
                "Context fits within budget, no compression needed"
            );
            return context;
        }

        debug!(
            current = current_tokens,
            budget = max_tokens,
            "Compressing context to fit budget"
        );

        // Sort sections by priority (highest priority = lowest enum value)
        let mut sections = context.sections;
        sections.sort_by_key(|s| s.priority);

        // Keep adding sections until we exceed budget
        let mut kept_sections: Vec<ContextSection> = Vec::new();
        let mut used_tokens = 0;
        let overhead = 100; // Reserve tokens for header/footer

        for section in sections {
            if used_tokens + section.estimated_tokens + overhead <= max_tokens {
                used_tokens += section.estimated_tokens;
                kept_sections.push(section);
            } else if section.priority == SectionPriority::Critical {
                // Always keep critical sections, even if over budget
                used_tokens += section.estimated_tokens;
                kept_sections.push(section);
            }
            // Skip non-critical sections that don't fit
        }

        // Rebuild text from kept sections
        let mut text = String::new();
        text.push_str(&crate::templates::header(&context.project_name));

        for section in &kept_sections {
            text.push_str(&crate::templates::section_header(&section.title));
            text.push_str(&section.content);
            text.push('\n');
        }

        text.push_str(&crate::templates::footer());

        let estimated_tokens = Self::estimate_tokens(&text);

        PortableContext {
            text,
            sections: kept_sections,
            estimated_tokens,
            project_name: context.project_name,
            generated_at: context.generated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_section(title: &str, content_len: usize, priority: SectionPriority) -> ContextSection {
        let content = "x".repeat(content_len);
        ContextSection {
            title: title.into(),
            content,
            priority,
            estimated_tokens: content_len.div_ceil(4),
        }
    }

    fn make_context(sections: Vec<ContextSection>) -> PortableContext {
        let text: String = sections.iter().map(|s| s.content.clone()).collect();
        let estimated_tokens = ContextCompressor::estimate_tokens(&text);
        PortableContext {
            text,
            sections,
            estimated_tokens,
            project_name: "Test".into(),
            generated_at: 0,
        }
    }

    #[test]
    fn estimate_tokens_basic() {
        assert_eq!(ContextCompressor::estimate_tokens(""), 0);
        assert_eq!(ContextCompressor::estimate_tokens("abcd"), 1);
        assert_eq!(ContextCompressor::estimate_tokens("abcdefgh"), 2);
    }

    #[test]
    fn no_compression_when_under_budget() {
        let ctx = make_context(vec![make_section("Arch", 100, SectionPriority::Critical)]);
        let compressed = ContextCompressor::compress(ctx, 10000);
        assert!(compressed.sections.len() == 1);
    }

    #[test]
    fn compression_drops_low_priority_first() {
        let ctx = make_context(vec![
            make_section("Critical", 400, SectionPriority::Critical),
            make_section("High", 400, SectionPriority::High),
            make_section("Low", 400, SectionPriority::Low),
        ]);
        // Budget can only fit ~2 sections
        let compressed = ContextCompressor::compress(ctx, 300);
        // Critical is always kept
        assert!(compressed.sections.iter().any(|s| s.title == "Critical"));
    }

    #[test]
    fn critical_sections_always_kept() {
        let ctx = make_context(vec![make_section(
            "Must Keep",
            2000,
            SectionPriority::Critical,
        )]);
        let compressed = ContextCompressor::compress(ctx, 100);
        assert_eq!(compressed.sections.len(), 1);
        assert_eq!(compressed.sections[0].title, "Must Keep");
    }
}
