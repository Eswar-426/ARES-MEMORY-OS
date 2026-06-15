use super::intent::QueryIntent;

pub struct QueryParser;

/// Common English words that should never be treated as code symbols
const STOP_WORDS: &[&str] = &[
    "explain", "provide", "trace", "find", "locate", "summarize", "identify",
    "describe", "analyze", "impact", "analysis", "list", "show", "what", "where",
    "which", "that", "from", "with", "this", "have", "does", "about", "into",
    "would", "could", "should", "including", "affected", "responsible",
    "comprehensive", "high", "level", "internal", "external", "downstream",
    "components", "steps", "sequence", "initial", "utilized", "coordinates",
    "execution", "extraction", "interfaces", "generating",
];

impl QueryParser {
    pub fn new() -> Self {
        Self
    }

    /// Extracts potential target files or symbols from the query string.
    pub fn extract_targets(&self, query: &str) -> Vec<String> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut targets = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for word in &words {
            // Trim ALL non-alphanumeric chars from edges (removes trailing dots, quotes, etc.)
            let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric());

            if word_clean.is_empty() || word_clean.len() < 3 {
                continue;
            }

            // Skip common English stop words
            if STOP_WORDS.contains(&word_clean.to_lowercase().as_str()) {
                continue;
            }

            // Check if it looks like a code symbol:
            // 1. Contains a path separator (., ::, /)
            // 2. Contains an underscore (snake_case)
            // 3. Has a mid-word uppercase letter (true CamelCase/PascalCase, not just "Explain")
            let has_path_sep = word_clean.contains('.') || word_clean.contains("::") || word_clean.contains('/');
            let has_underscore = word_clean.contains('_');
            let is_true_camel_case = Self::is_camel_or_pascal_case(word_clean);

            if has_path_sep || has_underscore || is_true_camel_case {
                if seen.insert(word_clean.to_string()) {
                    targets.push(word_clean.to_string());
                }
            }
        }

        // Fallback: if no strict technical symbols found, extract long non-stop words
        if targets.is_empty() {
            let mut long_words: Vec<&str> = words.iter()
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
                .filter(|w| w.len() > 5)
                .filter(|w| !STOP_WORDS.contains(&w.to_lowercase().as_str()))
                .collect();
            long_words.sort_by(|a, b| b.len().cmp(&a.len()));
            long_words.dedup();
            for w in long_words.into_iter().take(2) {
                targets.push(w.to_string());
            }
        }

        targets
    }

    /// Detects true CamelCase/PascalCase: requires an uppercase letter
    /// that is preceded by a lowercase letter, or vice versa (mid-word transition).
    /// This excludes plain English words like "Explain" or "Provide".
    fn is_camel_or_pascal_case(s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() < 4 {
            return false;
        }
        // Must start with uppercase (PascalCase like MemoryBuilder)
        if !chars[0].is_uppercase() {
            return false;
        }
        // Must have at least one lowercase-to-uppercase transition after position 0
        for i in 1..chars.len() {
            if chars[i].is_uppercase() && i > 0 && chars[i - 1].is_lowercase() {
                return true;
            }
        }
        false
    }
}
