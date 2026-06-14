use crate::retrieval::ContextSelector;
use crate::types::{ContextPackage, TokenBudget};
use ares_core::AresError;
use ares_store::Store;
use tracing::info;

pub struct ContextInjector {
    selector: ContextSelector,
}

impl ContextInjector {
    pub fn new(store: Store) -> Self {
        Self {
            selector: ContextSelector::new(store),
        }
    }

    pub async fn inject(
        &self,
        project_id: &str,
        prompt: &str,
        budget: TokenBudget,
    ) -> Result<ContextPackage, AresError> {
        let mut package = self
            .selector
            .build_package(project_id, prompt, budget)
            .await?;

        let assembler = PromptAssembler::new(budget);
        assembler.assemble(&mut package)?;

        Ok(package)
    }
}

pub struct PromptAssembler {
    budget: TokenBudget,
}

impl PromptAssembler {
    pub fn new(budget: TokenBudget) -> Self {
        Self { budget }
    }

    pub fn assemble(&self, package: &mut ContextPackage) -> Result<(), AresError> {
        let mut final_prompt = String::new();
        let budget_limit = self.budget.as_usize();

        // 1. PROJECT OVERVIEW
        final_prompt.push_str("====================\n");
        final_prompt.push_str("PROJECT OVERVIEW\n");
        final_prompt.push_str("====================\n");
        final_prompt.push_str(&format!("Project ID: {}\n\n", package.project_id));

        // 2. ARCHITECTURE
        final_prompt.push_str("====================\n");
        final_prompt.push_str("ARCHITECTURE\n");
        final_prompt.push_str("====================\n");
        for node in &package.architecture_nodes {
            let line = format!(
                "- [{:?}] {} ({})\n",
                node.node_type,
                node.label,
                node.file_path.as_deref().unwrap_or("")
            );
            if Self::estimate_tokens(&final_prompt) + Self::estimate_tokens(&line) > budget_limit {
                break;
            }
            final_prompt.push_str(&line);
        }
        final_prompt.push('\n');

        // 3. DECISIONS
        final_prompt.push_str("====================\n");
        final_prompt.push_str("DECISIONS\n");
        final_prompt.push_str("====================\n");
        for dec in &package.decisions {
            let line = format!("- [{}]: {}\n", dec.title, dec.decision_text);
            if Self::estimate_tokens(&final_prompt) + Self::estimate_tokens(&line) > budget_limit {
                break;
            }
            final_prompt.push_str(&line);
        }
        final_prompt.push('\n');

        // 4. KNOWN BUGS
        final_prompt.push_str("====================\n");
        final_prompt.push_str("KNOWN BUGS\n");
        final_prompt.push_str("====================\n");
        for bug in &package.bugs {
            let bug_desc = bug
                .properties
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let line = format!("- {}: {}\n", bug.label, bug_desc);
            if Self::estimate_tokens(&final_prompt) + Self::estimate_tokens(&line) > budget_limit {
                break;
            }
            final_prompt.push_str(&line);
        }
        final_prompt.push('\n');

        // 5. RELEVANT MEMORIES (FILES)
        final_prompt.push_str("====================\n");
        final_prompt.push_str("RELEVANT FILES\n");
        final_prompt.push_str("====================\n");
        for mem in &package.memories {
            let line = format!(
                "- {}: {}\n",
                mem.label,
                mem.file_path.as_deref().unwrap_or("")
            );
            if Self::estimate_tokens(&final_prompt) + Self::estimate_tokens(&line) > budget_limit {
                break;
            }
            final_prompt.push_str(&line);
        }
        final_prompt.push('\n');

        // 6. ORIGINAL PROMPT
        final_prompt.push_str("====================\n");
        final_prompt.push_str("USER PROMPT\n");
        final_prompt.push_str("====================\n");
        final_prompt.push_str(&package.original_prompt);
        final_prompt.push('\n');

        package.assembled_prompt = final_prompt.clone();
        package.estimated_tokens = Self::estimate_tokens(&final_prompt);

        info!("Assembled prompt with ~{} tokens", package.estimated_tokens);

        Ok(())
    }

    /// A rough heuristic: 1 token ~= 4 characters.
    fn estimate_tokens(text: &str) -> usize {
        text.len() / 4
    }
}
