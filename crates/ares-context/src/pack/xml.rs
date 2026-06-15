use crate::models::ContextPack;

pub trait ToXml {
    fn to_xml(&self) -> String;
}

impl ToXml for ContextPack {
    fn to_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<context_pack>\n");
        xml.push_str(&format!("  <query>{}</query>\n", self.query));
        xml.push_str(&format!("  <intent>{:?}</intent>\n", self.intent));
        xml.push_str(&format!("  <summary>{}</summary>\n", self.summary));
        
        xml.push_str("  <relevant_files>\n");
        for file in &self.relevant_files {
            xml.push_str(&format!("    <file>{}</file>\n", file));
        }
        xml.push_str("  </relevant_files>\n");

        xml.push_str("  <retrieval_explanation>\n");
        for node in &self.retrieval_explanation.selected_nodes {
            xml.push_str(&format!("    <selected_node>{}</selected_node>\n", node));
        }
        for reason in &self.retrieval_explanation.ranking_reasons {
            xml.push_str(&format!("    <ranking_reason>{}</ranking_reason>\n", reason));
        }
        xml.push_str("  </retrieval_explanation>\n");
        
        xml.push_str(&format!("  <confidence>{}</confidence>\n", self.confidence_score));
        xml.push_str(&format!("  <retrieval_latency_ms>{}</retrieval_latency_ms>\n", self.retrieval_time_ms));
        xml.push_str("</context_pack>\n");

        xml
    }
}
