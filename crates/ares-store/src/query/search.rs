// Placeholder — full-text search helpers for complex multi-field queries.
// The primary FTS implementation lives in SqliteMemoryRepository::search.
// This module provides utilities for query sanitization and snippet extraction.

/// Sanitize a user-provided FTS5 query string.
/// FTS5 queries can include special operators (AND, OR, NOT, phrases, prefixes).
/// We escape bare strings to prevent syntax errors.
pub fn sanitize_fts_query(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // If the query looks like a simple word(s) with no FTS operators,
    // wrap it for safe prefix search
    let has_operators = trimmed.contains('"')
        || trimmed.contains('(')
        || trimmed.contains(')')
        || trimmed.to_uppercase().contains(" AND ")
        || trimmed.to_uppercase().contains(" OR ")
        || trimmed.to_uppercase().contains(" NOT ");

    if has_operators {
        // Pass through as-is (user knows what they're doing)
        trimmed.to_string()
    } else {
        // Treat each word as a prefix search term
        trimmed
            .split_whitespace()
            .map(|word| format!("{word}*"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_query_becomes_prefix_search() {
        let q = sanitize_fts_query("authentication jwt");
        assert_eq!(q, "authentication* jwt*");
    }

    #[test]
    fn operator_query_passes_through() {
        let q = sanitize_fts_query("\"exact phrase\"");
        assert_eq!(q, "\"exact phrase\"");
    }

    #[test]
    fn empty_query_returns_empty() {
        assert_eq!(sanitize_fts_query("  "), "");
    }
}
