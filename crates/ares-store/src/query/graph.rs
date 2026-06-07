// Graph query helpers — complex traversal patterns used by context engine.
// The primary graph queries live in SqliteGraphRepository.
// This module provides helpers for building dynamic Cypher-like SQL patterns.

/// Maximum allowed traversal depth (enforced at query level).
pub const MAX_TRAVERSAL_DEPTH: u8 = 5;

/// Edge types that represent structural dependencies (used in impact analysis).
pub const STRUCTURAL_EDGE_TYPES: &[&str] = &[
    "imports",
    "depends_on",
    "calls",
    "implements",
    "defines",
];

/// Edge types that represent knowledge relationships.
pub const KNOWLEDGE_EDGE_TYPES: &[&str] = &[
    "impacts",
    "motivated_by",
    "fixed_by",
    "supersedes",
    "caused",
    "related_to",
];

/// Compute confidence decay for a given hop depth.
/// Base confidence = 1.0, decays by 0.1 per hop, minimum 0.1.
pub fn confidence_at_depth(depth: u8) -> f32 {
    (1.0_f32 - (depth as f32 * 0.1)).max(0.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_at_depth_0_is_1() {
        assert!((confidence_at_depth(0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn confidence_at_depth_5_is_half() {
        assert!((confidence_at_depth(5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn confidence_never_below_min() {
        assert_eq!(confidence_at_depth(20), 0.1);
    }
}
