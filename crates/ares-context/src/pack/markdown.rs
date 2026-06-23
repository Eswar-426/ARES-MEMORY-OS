use crate::models::ContextPack;

pub trait ToMarkdown {
    fn to_markdown(&self) -> String;
}

impl ToMarkdown for ContextPack {
    fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&"# ARES Context Pack\n\n".to_string());
        md.push_str(&format!("**Query:** {}\n", self.query));
        md.push_str(&format!("**Intent:** {:?}\n\n", self.intent));
        md.push_str(&format!("## Summary\n{}\n\n", self.summary));

        md.push_str("## Relevant Files\n");
        for file in &self.relevant_files {
            md.push_str(&format!("- `{}`\n", file));
        }
        md.push('\n');

        md.push_str("## Selected Nodes\n");
        for node in &self.retrieval_explanation.selected_nodes {
            md.push_str(&format!("- {}\n", node));
        }
        md.push('\n');

        md.push_str("## Ranking Reasons\n");
        for reason in &self.retrieval_explanation.ranking_reasons {
            md.push_str(&format!("- {}\n", reason));
        }
        md.push('\n');

        md.push_str(&format!("**Confidence:** {:.2}\n", self.confidence_score));
        md.push_str(&format!(
            "**Retrieval Latency:** {}ms\n",
            self.retrieval_time_ms
        ));

        md
    }
}
