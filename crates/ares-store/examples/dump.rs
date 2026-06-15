fn main() {
    let query = "Trace scanner dependencies Trace all internal and external dependencies utilized by the ares-scanner crate for analyzing the AST.";
    let words: Vec<&str> = query.split_whitespace().collect();
    let mut targets = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let stop_words: &[&str] = &[
        "explain", "provide", "trace", "find", "locate", "summarize", "identify",
        "describe", "analyze", "impact", "analysis", "list", "show", "what", "where",
        "which", "that", "from", "with", "this", "have", "does", "about", "into",
        "would", "could", "should", "including", "affected", "responsible",
        "comprehensive", "high", "level", "internal", "external", "downstream",
        "components", "steps", "sequence", "initial", "utilized", "coordinates",
        "execution", "extraction", "interfaces", "generating",
    ];

    for word in &words {
        // Trim ALL non-alphanumeric chars from edges (removes trailing dots, quotes, etc.)
        let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric());

        if word_clean.is_empty() || word_clean.len() < 3 {
            continue;
        }

        if stop_words.contains(&word_clean.to_lowercase().as_str()) {
            continue;
        }

        let has_path_sep = word_clean.contains('.') || word_clean.contains("::") || word_clean.contains('/');
        let has_underscore = word_clean.contains('_');
        let is_true_camel_case = {
            let chars: Vec<char> = word_clean.chars().collect();
            let mut res = false;
            if chars.len() >= 4 && chars[0].is_uppercase() {
                for i in 1..chars.len() {
                    if chars[i].is_uppercase() && i > 0 && chars[i - 1].is_lowercase() {
                        res = true;
                        break;
                    }
                }
            }
            res
        };

        if has_path_sep || has_underscore || is_true_camel_case {
            if seen.insert(word_clean.to_string()) {
                targets.push(word_clean.to_string());
            }
        }
    }

    if targets.is_empty() {
        let mut long_words: Vec<&str> = words.iter()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| w.len() > 5)
            .filter(|w| !stop_words.contains(&w.to_lowercase().as_str()))
            .collect();
        long_words.sort_by(|a, b| b.len().cmp(&a.len()));
        long_words.dedup();
        for w in long_words.into_iter().take(2) {
            targets.push(w.to_string());
        }
    }

    println!("Targets: {:?}", targets);
}
