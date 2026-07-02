use crate::core::response::RepositoryResponse;

pub struct PromptAssembler;

impl PromptAssembler {
    pub fn assemble(query: &str, response: &RepositoryResponse) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are ARES, an AI repository intelligence assistant. ");
        prompt.push_str("You answer questions based ONLY on the provided repository evidence.\n\n");

        prompt.push_str(&format!("USER QUESTION: {}\n\n", query));

        prompt.push_str("=== EVIDENCE ===\n");

        // Extract from ProcessedEvidenceBundle
        let ev = &response.evidence;

        if let Some(ref graph) = ev.graph {
            prompt.push_str("GRAPH EVIDENCE:\n");
            for node in &graph.nodes {
                prompt.push_str(&format!("- Node: {}\n", node));
            }
            for edge in &graph.edges {
                prompt.push_str(&format!("- Edge: {}\n", edge));
            }
            prompt.push('\n');
        }

        if let Some(ref code) = ev.code {
            prompt.push_str("CODE EVIDENCE:\n");
            for file in &code.files {
                prompt.push_str(&format!("- File: {}\n", file));
            }
            for func_str in &code.functions {
                prompt.push_str(&format!("Function: {}\n", func_str));
            }
            prompt.push('\n');
        }

        if let Some(ref git) = ev.git {
            prompt.push_str("GIT EVIDENCE:\n");
            for commit in &git.commits {
                prompt.push_str(&format!("- Commit: {}\n", commit));
            }
            for author in &git.authors {
                prompt.push_str(&format!("- Author: {}\n", author));
            }
            prompt.push('\n');
        }

        if let Some(ref arch) = ev.architecture {
            prompt.push_str("ARCHITECTURE EVIDENCE:\n");
            prompt.push_str(&format!("{:?}\n\n", arch));
        }

        prompt.push_str("=== END EVIDENCE ===\n\n");
        prompt.push_str(
            "Based on the evidence above, answer the user's question clearly and concisely. ",
        );
        prompt.push_str(
            "If the evidence is insufficient, state what is missing instead of guessing.",
        );

        prompt
    }
}
