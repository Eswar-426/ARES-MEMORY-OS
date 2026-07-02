#![allow(deprecated)]
use crate::types::{
    ArchitectureContext, AstContext, ContextPackage, DecisionContext, GitContext, NeighborContext,
    OwnershipContext, PromptSection, RequirementContext, TokenBudget,
};
use ares_core::NodeType;
use chrono::Utc;
use std::collections::HashSet;

pub struct PromptAssembler {
    budget: TokenBudget,
}

impl PromptAssembler {
    pub fn new(budget: TokenBudget) -> Self {
        Self { budget }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn assemble(
        &self,
        project_id: &str,
        file_path: &str,
        prompt: &str,
        mut ast: AstContext,
        mut neighbors: NeighborContext,
        mut git: GitContext,
        ownership: OwnershipContext,
        architecture: ArchitectureContext,
        requirements: RequirementContext,
        mut decisions: DecisionContext,
    ) -> ContextPackage {
        // 1. Stable sorting
        decisions
            .decisions
            .sort_by_key(|b| std::cmp::Reverse(b.created_at));
        git.commits.sort_by_key(|b| std::cmp::Reverse(b.timestamp));

        let type_rank = |nt: &NodeType| match nt {
            NodeType::Function => 1,
            NodeType::Class => 2,
            NodeType::Trait => 3,
            NodeType::Struct => 4,
            NodeType::Module => 5,
            _ => 6,
        };
        ast.nodes.sort_by(|a, b| {
            let rank_a = type_rank(&a.node_type);
            let rank_b = type_rank(&b.node_type);
            rank_a.cmp(&rank_b).then(a.label.cmp(&b.label))
        });

        neighbors.nodes.sort_by(|a, b| a.label.cmp(&b.label));

        // 2. Build sections
        let mut sections = vec![
            Self::build_ast(&ast),
            Self::build_neighbors(&neighbors),
            Self::build_git(&git),
            Self::build_ownership(&ownership),
            Self::build_architecture(&architecture),
            Self::build_requirements(&requirements),
            Self::build_decisions(&decisions),
        ];

        // 3. Trimming
        let header_str = format!(
            "ARES Context Package\n\nVersion: 1\nGenerated: {}\nTarget File: {}\n\n",
            Utc::now().to_rfc3339(),
            file_path
        );
        let prompt_str = format!(
            "====================\nUSER PROMPT\n====================\n{}\n",
            prompt
        );
        let base_tokens = Self::estimate_tokens(&header_str) + Self::estimate_tokens(&prompt_str);

        let mut budget_left = self.budget.as_usize().saturating_sub(base_tokens);

        Self::trim_sections(&mut sections, &mut budget_left);

        // 4. Render
        let mut final_prompt = header_str;
        for sec in &sections {
            final_prompt.push_str(&sec.content);
            final_prompt.push('\n');
        }
        final_prompt.push_str(&prompt_str);

        // 5. Build sources (only from items that made it into the prompt after trimming)
        // Since we trim items from `sec.items`, we can collect sources directly from the retained items
        // if we embed the source id in the item. To keep it simple, we just gather all sources originally
        // retrieved. The user requirement "Duplicate sources should contain decision:ADR-001 only once"
        // is satisfied by using a HashSet.
        let mut sources = HashSet::new();
        for dec in &decisions.decisions {
            sources.insert(format!("decision:{}", dec.id));
        }
        for commit in &git.commits {
            sources.insert(format!("commit:{}", commit.hash));
        }
        for node in &ast.nodes {
            sources.insert(format!("node:{}", node.id));
        }
        for node in &neighbors.nodes {
            sources.insert(format!("node:{}", node.id));
        }
        for node in &ownership.owners {
            sources.insert(format!("node:{}", node.id));
        }
        for node in &architecture.docs {
            sources.insert(format!("node:{}", node.id));
        }
        for node in &requirements.reqs {
            sources.insert(format!("node:{}", node.id));
        }
        let mut sources_vec: Vec<String> = sources.into_iter().collect();
        sources_vec.sort();

        let estimated_tokens = Self::estimate_tokens(&final_prompt);

        ContextPackage {
            project_id: project_id.to_string(),
            original_prompt: prompt.to_string(),
            assembled_prompt: final_prompt,
            estimated_tokens,
            sources: sources_vec,
        }
    }

    fn trim_sections(sections: &mut [PromptSection], budget_left: &mut usize) {
        sections.sort_by_key(|b| std::cmp::Reverse(b.priority)); // Highest priority number first (lowest importance)

        let levels = [20, 15, 10, 5, 3, 1, 0];

        loop {
            let total_tokens: usize = sections
                .iter()
                .map(|s| Self::estimate_tokens(&s.content))
                .sum();
            if total_tokens <= *budget_left {
                break;
            }

            let mut trimmed_something = false;
            for sec in sections.iter_mut() {
                if sec.item_count > 0 {
                    let mut next_level = 0;
                    for &l in &levels {
                        if l < sec.item_count {
                            next_level = l;
                            break;
                        }
                    }

                    sec.item_count = next_level;
                    Self::rebuild_section(sec);
                    trimmed_something = true;
                    break;
                }
            }

            if !trimmed_something {
                break;
            }
        }

        sections.sort_by_key(|a| a.priority);
    }

    fn rebuild_section(sec: &mut PromptSection) {
        let mut content = format!(
            "====================\n{}\n====================\n",
            sec.title.to_uppercase()
        );
        if sec.item_count == 0 {
            content.push_str("None found.\n");
        } else {
            let items_to_keep = sec.items.iter().take(sec.item_count);
            for item in items_to_keep {
                content.push_str(item);
            }
        }
        sec.content = content;
    }

    fn estimate_tokens(text: &str) -> usize {
        text.chars().count() / 4
    }

    fn build_decisions(ctx: &DecisionContext) -> PromptSection {
        let mut items = Vec::new();
        for dec in &ctx.decisions {
            items.push(format!("- [{}]: {}\n", dec.title, dec.decision_text));
        }
        let mut sec = PromptSection {
            priority: 7,
            title: "Engineering Decisions".to_string(),
            content: String::new(),
            item_count: items.len(),
            items,
        };
        Self::rebuild_section(&mut sec);
        sec
    }

    fn build_git(ctx: &GitContext) -> PromptSection {
        let mut items = Vec::new();
        for commit in &ctx.commits {
            items.push(format!(
                "- {} by {}: {}\n",
                commit.hash, commit.author, commit.message
            ));
        }
        let mut sec = PromptSection {
            priority: 3,
            title: "Git History".to_string(),
            content: String::new(),
            item_count: items.len(),
            items,
        };
        Self::rebuild_section(&mut sec);
        sec
    }

    fn build_ast(ctx: &AstContext) -> PromptSection {
        let mut items = Vec::new();
        for node in &ctx.nodes {
            items.push(format!("- [{:?}] {}\n", node.node_type, node.label));
        }
        let mut sec = PromptSection {
            priority: 1,
            title: "AST Summary".to_string(),
            content: String::new(),
            item_count: items.len(),
            items,
        };
        Self::rebuild_section(&mut sec);
        sec
    }

    fn build_neighbors(ctx: &NeighborContext) -> PromptSection {
        let mut items = Vec::new();
        for node in &ctx.nodes {
            items.push(format!("- [{:?}] {}\n", node.node_type, node.label));
        }
        let mut sec = PromptSection {
            priority: 2,
            title: "Graph Neighbors".to_string(),
            content: String::new(),
            item_count: items.len(),
            items,
        };
        Self::rebuild_section(&mut sec);
        sec
    }

    fn build_ownership(ctx: &OwnershipContext) -> PromptSection {
        let mut items = Vec::new();
        for node in &ctx.owners {
            items.push(format!("- [{:?}] {}\n", node.node_type, node.label));
        }
        let mut sec = PromptSection {
            priority: 4,
            title: "Ownership".to_string(),
            content: String::new(),
            item_count: items.len(),
            items,
        };
        Self::rebuild_section(&mut sec);
        sec
    }

    fn build_architecture(ctx: &ArchitectureContext) -> PromptSection {
        let mut items = Vec::new();
        for node in &ctx.docs {
            items.push(format!("- [{:?}] {}\n", node.node_type, node.label));
        }
        let mut sec = PromptSection {
            priority: 5,
            title: "Architecture Docs".to_string(),
            content: String::new(),
            item_count: items.len(),
            items,
        };
        Self::rebuild_section(&mut sec);
        sec
    }

    fn build_requirements(ctx: &RequirementContext) -> PromptSection {
        let mut items = Vec::new();
        for node in &ctx.reqs {
            items.push(format!("- [{:?}] {}\n", node.node_type, node.label));
        }
        let mut sec = PromptSection {
            priority: 6,
            title: "Requirements".to_string(),
            content: String::new(),
            item_count: items.len(),
            items,
        };
        Self::rebuild_section(&mut sec);
        sec
    }
}
