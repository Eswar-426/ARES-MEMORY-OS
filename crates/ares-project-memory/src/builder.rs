//! MemoryBuilder — Orchestrates all analyzers and trackers to produce a ProjectSnapshot.

use crate::analyzer::{ArchitectureAnalyzer, DependencyAnalyzer, FolderAnalyzer, LanguageAnalyzer};
use crate::tracker::ChangeTracker;
use crate::types::*;
use ares_core::{AresError, Project, ProjectId};
use ares_store::repositories::decision::SqliteDecisionRepository;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;
use ares_store::repositories::project::SqliteProjectRepository;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

pub struct MemoryBuilder {
    project_repo: Arc<SqliteProjectRepository>,
    memory_repo: Arc<SqliteMemoryRepository>,
    decision_repo: Arc<SqliteDecisionRepository>,
    _graph_repo: Arc<SqliteGraphRepository>,
}

impl MemoryBuilder {
    pub fn new(
        project_repo: Arc<SqliteProjectRepository>,
        memory_repo: Arc<SqliteMemoryRepository>,
        decision_repo: Arc<SqliteDecisionRepository>,
        _graph_repo: Arc<SqliteGraphRepository>,
    ) -> Self {
        Self {
            project_repo,
            memory_repo,
            decision_repo,
            _graph_repo,
        }
    }

    /// Build a complete project snapshot from filesystem analysis + stored memories.
    pub fn build_snapshot(&self, project: &Project) -> Result<ProjectSnapshot, AresError> {
        let root = Path::new(&project.root_path);
        info!(project = %project.name, root = %root.display(), "Building project snapshot");

        // 1. Filesystem analysis
        let architecture = ArchitectureAnalyzer::analyze(root);
        let languages = LanguageAnalyzer::analyze(root);
        let dependencies = DependencyAnalyzer::analyze(root);
        let folder_structure = FolderAnalyzer::analyze(root, 3);

        // 2. Memory store analysis
        let tracker = ChangeTracker::new(self.memory_repo.clone(), self.decision_repo.clone());
        let decisions = tracker.get_decisions(&project.id).unwrap_or_default();
        let features = tracker.get_features(&project.id).unwrap_or_default();
        let bugs = tracker.get_bugs(&project.id).unwrap_or_default();
        let recent_changes = tracker
            .get_recent_changes(&project.id, 50)
            .unwrap_or_default();

        // 3. Compute stats
        let memory_counts = self
            .project_repo
            .get_memory_counts(&project.id)
            .unwrap_or_default();
        let total_files: u32 = languages.iter().map(|l| l.file_count).sum();
        let total_lines: u64 = languages.iter().map(|l| l.line_count).sum();
        let total_memories: u64 = memory_counts.values().sum();
        let total_decisions = decisions.len() as u64;

        let stats = ProjectStats {
            total_files,
            total_lines,
            total_memories,
            total_decisions,
            total_graph_nodes: 0, // TODO: add graph_repo.count_nodes()
            total_graph_edges: 0, // TODO: add graph_repo.count_edges()
            memory_counts_by_type: memory_counts,
        };

        let snapshot = ProjectSnapshot {
            project_id: project.id.as_str().to_string(),
            name: project.name.clone(),
            description: project.description.clone(),
            root_path: project.root_path.clone(),
            architecture,
            languages,
            frameworks: Self::detect_frameworks(&dependencies),
            dependencies,
            folder_structure,
            api_endpoints: vec![], // Populated from graph nodes in future
            decisions,
            features,
            bugs,
            recent_changes,
            stats,
            created_at: chrono::Utc::now().timestamp_micros(),
            snapshot_version: 1,
        };

        info!(
            project = %project.name,
            files = snapshot.stats.total_files,
            languages = snapshot.languages.len(),
            dependencies = snapshot.dependencies.len(),
            decisions = snapshot.decisions.len(),
            "Project snapshot built"
        );

        Ok(snapshot)
    }

    /// Build a snapshot from a project ID (looks up the project first).
    pub fn build_snapshot_by_id(
        &self,
        project_id: &ProjectId,
    ) -> Result<ProjectSnapshot, AresError> {
        let project = self
            .project_repo
            .get_by_id(project_id)?
            .ok_or_else(|| AresError::not_found("project", project_id.as_str()))?;
        self.build_snapshot(&project)
    }

    /// Detect frameworks from dependency names.
    fn detect_frameworks(deps: &[DependencyInfo]) -> Vec<String> {
        let mut frameworks = Vec::new();
        let known = [
            ("react", "React"),
            ("react-dom", "React"),
            ("next", "Next.js"),
            ("vue", "Vue.js"),
            ("angular", "Angular"),
            ("svelte", "Svelte"),
            ("express", "Express"),
            ("fastify", "Fastify"),
            ("axum", "Axum"),
            ("actix-web", "Actix"),
            ("rocket", "Rocket"),
            ("tokio", "Tokio"),
            ("django", "Django"),
            ("flask", "Flask"),
            ("fastapi", "FastAPI"),
            ("gin", "Gin"),
            ("spring-boot", "Spring Boot"),
            ("tailwindcss", "Tailwind CSS"),
            ("vite", "Vite"),
            ("webpack", "Webpack"),
            ("jest", "Jest"),
            ("pytest", "Pytest"),
        ];

        for dep in deps {
            for (name, framework) in &known {
                if dep.name.to_lowercase() == *name
                    && dep.dep_type == DependencyType::Runtime
                    && !frameworks.contains(&framework.to_string())
                {
                    frameworks.push(framework.to_string());
                }
            }
        }

        frameworks
    }
}
