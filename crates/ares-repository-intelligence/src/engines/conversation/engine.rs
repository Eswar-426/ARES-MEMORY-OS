use std::sync::Arc;

use crate::core::context::RepositoryContext;
use crate::core::response::RepositoryResponse;
use crate::planner::pipeline::ExecutionPlanner;
use ares_core::inference::InferenceEngine;

use super::actions::Action;
use super::prompt::PromptAssembler;

pub struct ConversationResponse {
    pub answer: String,
    pub response: RepositoryResponse,
    pub actions: Vec<Action>,
}

pub struct ConversationEngine<'a> {
    planner: &'a ExecutionPlanner<'a>,
    inference: Arc<dyn InferenceEngine>,
}

impl<'a> ConversationEngine<'a> {
    pub fn new(planner: &'a ExecutionPlanner<'a>, inference: Arc<dyn InferenceEngine>) -> Self {
        Self { planner, inference }
    }

    #[tracing::instrument(name = "ConversationEngine::ask", skip(self, context))]
    pub async fn ask(
        &self,
        query: &str,
        context: &mut RepositoryContext,
    ) -> Result<ConversationResponse, ares_core::AresError> {
        // Ensure the context request query is updated
        context.request.query = query.to_string();

        // 1. Run ExecutionPlanner
        let mut response = self.planner.execute(context).await;

        // 2. Prompt Assembly
        let prompt = PromptAssembler::assemble(query, &response);

        // 3. Inference Provider
        let answer_val = self.inference.complete(&prompt).await?;

        let answer = if let Some(content) = answer_val
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|c| c.as_str())
        {
            content.to_string()
        } else if let Some(text) = answer_val.as_str() {
            text.to_string()
        } else {
            answer_val.to_string()
        };

        // Populate Answer in Response
        response.answer = Some(answer.clone());

        // 4. Generate Citations
        response.citations = self.generate_citations(&response);

        // 5. Generate Suggested Actions
        let actions = self.generate_actions(&response);
        response.actions = actions
            .iter()
            .map(|a| crate::core::response::ActionSuggestion {
                command: format!("{:?}", a.action_type),
                label: a.title.clone(),
                arguments: vec![a.payload.to_string()],
            })
            .collect();

        Ok(ConversationResponse {
            answer,
            response,
            actions,
        })
    }

    fn generate_actions(&self, response: &RepositoryResponse) -> Vec<Action> {
        let mut actions = Vec::new();

        let ev = &response.evidence;
        if let Some(ref graph) = ev.graph {
            if let Some(node) = graph.nodes.first() {
                actions.push(Action::open_graph(node));
                actions.push(Action::run_impact(node));
                actions.push(Action::traceability(node));
            }
        }

        if let Some(ref code) = ev.code {
            if let Some(file) = code.files.first() {
                actions.push(Action::open_file(file));
            }
        }

        actions
    }

    fn generate_citations(
        &self,
        response: &RepositoryResponse,
    ) -> Vec<crate::core::response::Citation> {
        let mut citations = Vec::new();
        let ev = &response.evidence;

        if let Some(ref graph) = ev.graph {
            for node in &graph.nodes {
                citations.push(crate::core::response::Citation {
                    kind: "Node".to_string(),
                    id: node.clone(),
                    title: format!("Graph Node: {}", node),
                    location: None,
                });
            }
        }

        if let Some(ref code) = ev.code {
            for file in &code.files {
                citations.push(crate::core::response::Citation {
                    kind: "File".to_string(),
                    id: file.clone(),
                    title: format!("Source File: {}", file),
                    location: Some(file.clone()),
                });
            }
        }

        if let Some(ref git) = ev.git {
            for commit in &git.commits {
                citations.push(crate::core::response::Citation {
                    kind: "Commit".to_string(),
                    id: commit.clone(),
                    title: format!("Commit: {}", commit),
                    location: None,
                });
            }
        }

        citations
    }
}
