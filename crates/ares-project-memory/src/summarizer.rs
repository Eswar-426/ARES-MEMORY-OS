use ares_core::{AresError, Project};
use crate::types::{ProjectSnapshot};
use ares_core::types::project::ProjectFingerprint;

pub enum SummaryTrigger {
    InitialScan,
    Manual,
    SignificantChange,
}

pub struct RepositorySummarizer;

impl RepositorySummarizer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compute_fingerprint(snapshot: &ProjectSnapshot) -> ProjectFingerprint {
        let total_files = snapshot.stats.total_files as usize;
        let mut languages: Vec<String> = snapshot.languages.iter().map(|l| l.language.clone()).collect();
        languages.sort();

        let mut all_files = String::new();
        // Since we don't have all files, we use the total numbers to derive a hash
        all_files.push_str(&format!("{}_{}_{}_{}", total_files, snapshot.stats.total_lines, snapshot.stats.total_memories, snapshot.stats.total_decisions));
        let hash = blake3::hash(all_files.as_bytes()).to_hex().to_string();

        ProjectFingerprint {
            total_files,
            languages,
            crates: 0, // Could be extracted from dependencies
            modules: 0, // Could be extracted from folder_structure
            hash,
        }
    }

    pub fn should_regenerate(prev: Option<&ProjectFingerprint>, current: &ProjectFingerprint) -> Option<SummaryTrigger> {
        match prev {
            None => Some(SummaryTrigger::InitialScan),
            Some(p) => {
                let diff_files = (p.total_files as isize - current.total_files as isize).abs() as usize;
                let percent_change = if p.total_files == 0 { 100.0 } else { (diff_files as f64 / p.total_files as f64) * 100.0 };
                
                if percent_change > 15.0 {
                    Some(SummaryTrigger::SignificantChange)
                } else if p.hash != current.hash {
                    // Small changes might just be a manual trigger later if user forces it.
                    None
                } else {
                    None
                }
            }
        }
    }

    pub async fn generate_summary(
        &self, 
        project: &Project, 
        snapshot: &ProjectSnapshot, 
        _trigger: SummaryTrigger,
        _provider: &dyn ares_extractor::provider::ExtractorProvider
    ) -> Result<String, AresError> {
        // Here we format the snapshot into structured markdown.
        // We simulate the LLM call using the generic provider if needed,
        // or for v0.9 we just synthesize the markdown directly based on the Snapshot!
        // The user said: "Generate structured markdown... Structured summaries are easier for agents to consume."

        let mut markdown = String::new();
        markdown.push_str(&format!("# Repository Summary: {}\n\n", project.name));
        
        markdown.push_str("## Purpose\n");
        markdown.push_str(&format!("{}\n\n", if project.description.is_empty() { "No description provided." } else { &project.description }));

        markdown.push_str("## Architecture\n");
        if snapshot.architecture.components.is_empty() {
            markdown.push_str("No major architecture components detected.\n\n");
        } else {
            markdown.push_str("### Components\n");
            for component in &snapshot.architecture.components {
                markdown.push_str(&format!("- **{}**: {}\n", component.name, component.description));
            }
            markdown.push_str("\n");
        }

        markdown.push_str("## Languages\n");
        for lang in &snapshot.languages {
            let percentage = if snapshot.stats.total_lines > 0 {
                (lang.line_count as f64 / snapshot.stats.total_lines as f64) * 100.0
            } else {
                0.0
            };
            markdown.push_str(&format!("- {} {:.1}%\n", lang.language, percentage));
        }
        markdown.push_str("\n");

        markdown.push_str("## Key Folders\n");
        for folder in snapshot.folder_structure.children.iter().take(5) {
            markdown.push_str(&format!("### {}\n", folder.name));
            markdown.push_str(&format!("Contains {} files.\n\n", folder.file_count));
        }

        markdown.push_str("## Recommended Next Steps\n");
        markdown.push_str("- Review automatically generated context.\n");
        markdown.push_str("- Build planner integration.\n");

        Ok(markdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProjectStats;

    #[test]
    fn test_compute_fingerprint() {
        let mut snapshot = ProjectSnapshot {
            project_id: "test".to_string(),
            name: "Test Project".to_string(),
            description: "".to_string(),
            root_path: "/".to_string(),
            architecture: crate::types::ArchitectureProfile { 
                style: crate::types::ArchitectureStyle::Unknown,
                components: vec![],
                patterns: vec![],
                entry_points: vec![],
            },
            languages: vec![
                crate::types::LanguageProfile { language: "Rust".to_string(), file_count: 10, line_count: 500, percentage: 100.0 }
            ],
            frameworks: vec![],
            dependencies: vec![],
            folder_structure: crate::types::FolderTree::new_dir("root"),
            api_endpoints: vec![],
            decisions: vec![],
            decision_coverage: None,
            requirement_coverage: None,
            requirements: vec![],
            features: vec![],
            bugs: vec![],
            recent_changes: vec![],
            stats: ProjectStats {
                total_files: 10,
                total_lines: 500,
                ..Default::default()
            },
            created_at: 0,
            snapshot_version: 1,
        };

        let fingerprint = RepositorySummarizer::compute_fingerprint(&snapshot);
        assert_eq!(fingerprint.total_files, 10);
        assert_eq!(fingerprint.languages, vec!["Rust".to_string()]);
        assert!(!fingerprint.hash.is_empty());
    }

    #[test]
    fn test_should_regenerate() {
        let fp1 = ProjectFingerprint {
            total_files: 10,
            languages: vec!["Rust".to_string()],
            crates: 0,
            modules: 0,
            hash: "abc".to_string(),
        };

        let fp2 = ProjectFingerprint {
            total_files: 12, // 20% change
            languages: vec!["Rust".to_string()],
            crates: 0,
            modules: 0,
            hash: "def".to_string(),
        };

        let fp3 = ProjectFingerprint {
            total_files: 10,
            languages: vec!["Rust".to_string()],
            crates: 0,
            modules: 0,
            hash: "abc".to_string(),
        };

        assert!(matches!(RepositorySummarizer::should_regenerate(None, &fp1), Some(SummaryTrigger::InitialScan)));
        assert!(matches!(RepositorySummarizer::should_regenerate(Some(&fp1), &fp2), Some(SummaryTrigger::SignificantChange)));
        assert!(matches!(RepositorySummarizer::should_regenerate(Some(&fp1), &fp3), None));
    }
}
