pub struct ContextManager {
    max_tokens: usize,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(4096)
    }
}

impl ContextManager {
    #[allow(dead_code)]
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }

    #[allow(dead_code)]
    pub fn estimate_tokens(&self, content: &str) -> usize {
        // Simple heuristic: 1 token ~ 4 chars. Ensure minimum 1 token for non-empty string
        if content.is_empty() {
            0
        } else {
            (content.len() / 4).max(1)
        }
    }

    #[allow(dead_code)]
    pub fn build_context(
        &self,
        prompt: &str,
        memories: &[String],
        system_instructions: &str,
    ) -> anyhow::Result<String> {
        let sys_tokens = self.estimate_tokens(system_instructions);
        let prompt_tokens = self.estimate_tokens(prompt);

        if sys_tokens + prompt_tokens > self.max_tokens {
            anyhow::bail!("Prompt and system instructions exceed maximum context window");
        }

        let mut context = String::new();
        context.push_str(system_instructions);
        context.push('\n');

        let mut current_tokens = sys_tokens + prompt_tokens;

        // Deduplicate memories using a HashSet
        let mut seen = std::collections::HashSet::new();

        context.push_str("=== Relevant Context ===\n");
        for memory in memories {
            if !seen.insert(memory.clone()) {
                continue; // Skip duplicate
            }

            let tokens = self.estimate_tokens(memory);
            if current_tokens + tokens <= self.max_tokens {
                context.push_str(memory);
                context.push('\n');
                current_tokens += tokens;
            } else {
                break;
            }
        }

        context.push_str("=== User Prompt ===\n");
        context.push_str(prompt);

        Ok(context)
    }
}
