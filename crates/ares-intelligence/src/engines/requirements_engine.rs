use chrono::Utc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use ares_candidates::{
    Candidate, CandidateConfidence, CandidateSource, CandidateStatus, CandidateType,
};
use ares_core::{GraphNode, NodeType};

pub struct RequirementCandidateEngine {
    project_id: String,
}

impl RequirementCandidateEngine {
    pub fn new(project_id: String) -> Self {
        Self { project_id }
    }

    /// Evaluates commit GraphNodes to heuristically propose requirement candidates.
    pub fn evaluate_commits(&self, nodes: &[GraphNode]) -> Vec<Candidate> {
        let mut candidates_map: HashMap<String, CandidateBuilder> = HashMap::new();

        for node in nodes {
            if node.node_type != NodeType::Commit {
                continue;
            }

            let subject = node
                .properties
                .get("subject")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let hash = node
                .properties
                .get("hash")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let msg_lower = subject.to_lowercase();

            // Extract feature keywords
            if msg_lower.contains("feat:")
                || msg_lower.contains("implement:")
                || msg_lower.contains("support:")
                || msg_lower.contains("add:")
            {
                // Extremely simple clustering: first line is the requirement
                let title = subject.lines().next().unwrap_or("").to_string();

                let builder = candidates_map
                    .entry(title.clone())
                    .or_insert_with(|| CandidateBuilder::new(&self.project_id, &title));

                builder.add_source(CandidateSource {
                    id: Uuid::new_v4().to_string(),
                    candidate_id: builder.id.clone(),
                    source_type: "commit".to_string(),
                    source_id: hash.to_string(),
                    confidence: 1.0,
                });
            }
        }

        candidates_map.into_values().map(|b| b.build()).collect()
    }

    /// Evaluates workspace structure (e.g., Cargo workspace crates or top-level dirs)
    pub fn evaluate_workspace_boundaries(&self, directories: &[String]) -> Vec<Candidate> {
        let mut candidates = Vec::new();

        for dir in directories {
            if dir.contains("crates/")
                || dir.contains("packages/")
                || dir.contains("docs/requirements")
            {
                let title = format!("Module: {}", dir);

                let mut builder = CandidateBuilder::new(&self.project_id, &title);
                builder.add_source(CandidateSource {
                    id: Uuid::new_v4().to_string(),
                    candidate_id: builder.id.clone(),
                    source_type: "directory".to_string(),
                    source_id: dir.clone(),
                    confidence: 1.0,
                });

                candidates.push(builder.build());
            }
        }

        candidates
    }
}

struct CandidateBuilder {
    id: String,
    project_id: String,
    title: String,
    sources: Vec<CandidateSource>,
    source_types: HashSet<String>,
}

impl CandidateBuilder {
    fn new(project_id: &str, title: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            title: title.to_string(),
            sources: Vec::new(),
            source_types: HashSet::new(),
        }
    }

    fn add_source(&mut self, source: CandidateSource) {
        self.source_types.insert(source.source_type.clone());
        self.sources.push(source);
    }

    fn build(self) -> Candidate {
        let now = Utc::now().timestamp_millis();

        let confidence = CandidateConfidence {
            evidence_count: self.sources.len() as u32,
            source_diversity: self.source_types.len() as u32,
            temporal_consistency: 1.0, // Simplified for V1
            cluster_strength: 1.0,     // Simplified for V1
        };

        Candidate {
            id: self.id,
            project_id: self.project_id,
            title: self.title.clone(),
            description: format!(
                "Automatically proposed based on {} pieces of evidence.",
                self.sources.len()
            ),
            candidate_type: CandidateType::Requirement,
            decision_category: None,
            architecture_category: None,
            traceability_category: None,
            source_endpoint: None,
            target_endpoint: None,
            traceability_strength: None,
            ownership_domains: Vec::new(),
            dependent_components: Vec::new(),
            status: CandidateStatus::Proposed,
            confidence,
            bootstrap_metadata: None,
            created_at: now,
            updated_at: now,
        }
    }
}
