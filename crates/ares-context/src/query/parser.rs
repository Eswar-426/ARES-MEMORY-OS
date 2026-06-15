use super::intent::QueryIntent;

pub struct QueryParser;

impl QueryParser {
    pub fn new() -> Self {
        Self
    }

    /// Extracts potential target files or symbols from the query string.
    pub fn extract_targets(&self, query: &str) -> Vec<String> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut targets = Vec::new();

        for word in words {
            // Very naive heuristic: looks like a file or a code symbol (camelCase / snake_case)
            let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '_');
            if word_clean.contains('.') || word_clean.contains('_') || word_clean.contains("::") {
                targets.push(word_clean.to_string());
            }
        }

        targets
    }
}
