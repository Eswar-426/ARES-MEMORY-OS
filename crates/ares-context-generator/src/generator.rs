//! ContextGenerator — transforms ProjectSnapshot into portable AI context.

use crate::compressor::ContextCompressor;
use crate::summarizer::MemorySummarizer;
use crate::templates;
use crate::types::*;
use ares_project_memory::types::ProjectSnapshot;
use tracing::info;

pub struct ContextGenerator;

impl ContextGenerator {
    /// Generate full portable context from a project snapshot.
    pub fn generate(snapshot: &ProjectSnapshot) -> PortableContext {
        let sections = vec![
            Self::build_architecture_section(snapshot),
            Self::build_state_section(snapshot),
            Self::build_decisions_section(snapshot),
            Self::build_recent_changes_section(snapshot),
            Self::build_features_section(snapshot),
            Self::build_bugs_section(snapshot),
            Self::build_dependencies_section(snapshot),
        ];
        // Build full text
        let mut text = String::new();
        text.push_str(&templates::header(&snapshot.name));

        for section in &sections {
            text.push_str(&templates::section_header(&section.title));
            text.push_str(&section.content);
            text.push('\n');
        }

        text.push_str(&templates::footer());

        let estimated_tokens = ContextCompressor::estimate_tokens(&text);

        info!(
            project = %snapshot.name,
            tokens = estimated_tokens,
            sections = sections.len(),
            "Generated portable context"
        );

        PortableContext {
            text,
            sections,
            estimated_tokens,
            project_name: snapshot.name.clone(),
            generated_at: chrono::Utc::now().timestamp_micros(),
        }
    }

    /// Generate context focused on a specific query/topic.
    pub fn generate_focused(snapshot: &ProjectSnapshot, _focus: &str) -> PortableContext {
        let full = Self::generate(snapshot);
        // For focused context, compress to a smaller budget
        ContextCompressor::compress(full, 4000)
    }

    /// Generate context compressed to fit a specific token budget.
    pub fn generate_for_budget(snapshot: &ProjectSnapshot, max_tokens: usize) -> PortableContext {
        let full = Self::generate(snapshot);
        ContextCompressor::compress(full, max_tokens)
    }

    // ─── Section builders ───────────────────────────────────────

    fn build_architecture_section(snapshot: &ProjectSnapshot) -> ContextSection {
        let mut content = String::new();

        content.push_str(&templates::kv(
            "Architecture",
            &format!("{:?}", snapshot.architecture.style),
        ));

        if !snapshot.languages.is_empty() {
            let langs: Vec<String> = snapshot
                .languages
                .iter()
                .take(5)
                .map(|l| format!("{} ({:.0}%)", l.language, l.percentage))
                .collect();
            content.push_str(&templates::kv("Languages", &langs.join(", ")));
        }

        if !snapshot.frameworks.is_empty() {
            content.push_str(&templates::kv(
                "Frameworks",
                &snapshot.frameworks.join(", "),
            ));
        }

        if !snapshot.architecture.components.is_empty() {
            content.push_str(&templates::kv(
                "Components",
                &format!("{} modules", snapshot.architecture.components.len()),
            ));
            let comp_names: Vec<String> = snapshot
                .architecture
                .components
                .iter()
                .take(15)
                .map(|c| format!("{} ({})", c.name, c.component_type))
                .collect();
            content.push_str(&templates::bullet_list(&comp_names));
        }

        if !snapshot.architecture.patterns.is_empty() {
            content.push_str(&templates::kv(
                "Patterns",
                &snapshot.architecture.patterns.join(", "),
            ));
        }

        let estimated_tokens = ContextCompressor::estimate_tokens(&content);
        ContextSection {
            title: "Architecture".into(),
            content,
            priority: SectionPriority::Critical,
            estimated_tokens,
        }
    }

    fn build_state_section(snapshot: &ProjectSnapshot) -> ContextSection {
        let mut content = String::new();

        content.push_str(&templates::kv("Project", &snapshot.name));
        content.push_str(&templates::kv("Description", &snapshot.description));
        content.push_str(&templates::kv(
            "Total Files",
            &snapshot.stats.total_files.to_string(),
        ));
        content.push_str(&templates::kv(
            "Total Lines",
            &snapshot.stats.total_lines.to_string(),
        ));
        content.push_str(&templates::kv(
            "Memories Stored",
            &snapshot.stats.total_memories.to_string(),
        ));
        content.push_str(&templates::kv(
            "Decisions Made",
            &snapshot.stats.total_decisions.to_string(),
        ));

        let estimated_tokens = ContextCompressor::estimate_tokens(&content);
        ContextSection {
            title: "Current State".into(),
            content,
            priority: SectionPriority::Critical,
            estimated_tokens,
        }
    }

    fn build_decisions_section(snapshot: &ProjectSnapshot) -> ContextSection {
        let items = MemorySummarizer::summarize_decisions(&snapshot.decisions);
        let content = templates::bullet_list(&items);
        let estimated_tokens = ContextCompressor::estimate_tokens(&content);
        ContextSection {
            title: "Important Decisions".into(),
            content,
            priority: SectionPriority::High,
            estimated_tokens,
        }
    }

    fn build_recent_changes_section(snapshot: &ProjectSnapshot) -> ContextSection {
        let items = MemorySummarizer::summarize_changes(&snapshot.recent_changes);
        let content = templates::bullet_list(&items);
        let estimated_tokens = ContextCompressor::estimate_tokens(&content);
        ContextSection {
            title: "Recent Work".into(),
            content,
            priority: SectionPriority::High,
            estimated_tokens,
        }
    }

    fn build_features_section(snapshot: &ProjectSnapshot) -> ContextSection {
        let (summary, items) = MemorySummarizer::summarize_features(&snapshot.features);
        let mut content = format!("{}\n", summary);
        if !items.is_empty() {
            content.push_str(&templates::bullet_list(&items));
        }
        let estimated_tokens = ContextCompressor::estimate_tokens(&content);
        ContextSection {
            title: "Features".into(),
            content,
            priority: SectionPriority::Medium,
            estimated_tokens,
        }
    }

    fn build_bugs_section(snapshot: &ProjectSnapshot) -> ContextSection {
        let (summary, items) = MemorySummarizer::summarize_bugs(&snapshot.bugs);
        let mut content = format!("{}\n", summary);
        if !items.is_empty() {
            content.push_str(&templates::bullet_list(&items));
        }
        let estimated_tokens = ContextCompressor::estimate_tokens(&content);
        ContextSection {
            title: "Known Issues".into(),
            content,
            priority: SectionPriority::Medium,
            estimated_tokens,
        }
    }

    fn build_dependencies_section(snapshot: &ProjectSnapshot) -> ContextSection {
        let mut content = String::new();

        if snapshot.dependencies.is_empty() {
            content.push_str("No dependencies tracked.\n");
        } else {
            let runtime: Vec<String> = snapshot
                .dependencies
                .iter()
                .filter(|d| d.dep_type == ares_project_memory::types::DependencyType::Runtime)
                .take(20)
                .map(|d| format!("{} {}", d.name, d.version))
                .collect();

            content.push_str(&format!(
                "{} total dependencies ({} runtime)\n",
                snapshot.dependencies.len(),
                runtime.len()
            ));
            if !runtime.is_empty() {
                content.push_str("Key runtime dependencies:\n");
                content.push_str(&templates::bullet_list(&runtime));
            }
        }

        let estimated_tokens = ContextCompressor::estimate_tokens(&content);
        ContextSection {
            title: "Dependencies".into(),
            content,
            priority: SectionPriority::Low,
            estimated_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_project_memory::types::*;

    fn make_snapshot() -> ProjectSnapshot {
        ProjectSnapshot {
            project_id: "proj_1".into(),
            name: "ARES MemoryOS".into(),
            description: "Universal AI Memory Layer".into(),
            root_path: "/projects/ares".into(),
            architecture: ArchitectureProfile {
                style: ArchitectureStyle::Modular,
                components: vec![ComponentInfo {
                    name: "ares-core".into(),
                    path: "crates/ares-core".into(),
                    component_type: "crate".into(),
                    description: "Core types".into(),
                }],
                patterns: vec!["Cargo workspace".into()],
                entry_points: vec!["src/main.rs".into()],
            },
            languages: vec![
                LanguageProfile {
                    language: "Rust".into(),
                    file_count: 50,
                    line_count: 25000,
                    percentage: 85.0,
                },
                LanguageProfile {
                    language: "TypeScript".into(),
                    file_count: 10,
                    line_count: 4000,
                    percentage: 15.0,
                },
            ],
            frameworks: vec!["Axum".into(), "Tokio".into()],
            dependencies: vec![],
            folder_structure: FolderTree::new_dir("root"),
            api_endpoints: vec![],
            decisions: vec![],
            decision_coverage: None,
            requirement_coverage: None,
            requirements: vec![],
            features: vec![],
            bugs: vec![],
            recent_changes: vec![ChangeRecord {
                change_type: ChangeType::DecisionMade,
                description: "Chose SQLite for storage".into(),
                files_affected: vec![],
                timestamp: 1000,
            }],
            stats: ProjectStats {
                total_files: 60,
                total_lines: 29000,
                total_memories: 15,
                total_decisions: 1,
                ..Default::default()
            },
            created_at: 0,
            snapshot_version: 1,
        }
    }

    #[test]
    fn generate_produces_non_empty_context() {
        let snapshot = make_snapshot();
        let ctx = ContextGenerator::generate(&snapshot);

        assert!(!ctx.text.is_empty());
        assert!(ctx.estimated_tokens > 0);
        assert!(ctx.text.contains("ARES MemoryOS"));
        assert!(ctx.text.contains("Architecture"));
        assert!(ctx.text.contains("Modular"));
        assert!(ctx.text.contains("Rust"));
    }

    #[test]
    fn generate_includes_all_sections() {
        let snapshot = make_snapshot();
        let ctx = ContextGenerator::generate(&snapshot);

        assert!(ctx.sections.len() >= 5);
        let titles: Vec<&str> = ctx.sections.iter().map(|s| s.title.as_str()).collect();
        assert!(titles.contains(&"Architecture"));
        assert!(titles.contains(&"Current State"));
        assert!(titles.contains(&"Important Decisions"));
    }

    #[test]
    fn generate_for_budget_compresses() {
        let snapshot = make_snapshot();
        let ctx = ContextGenerator::generate_for_budget(&snapshot, 500);

        // Should still have critical sections
        assert!(ctx
            .sections
            .iter()
            .any(|s| s.priority == SectionPriority::Critical));
    }
}
