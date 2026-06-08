//! Vector similarity computation.
//!
//! Provides numerically stable, allocation-efficient cosine similarity.
//! The current implementation is brute-force; the API is designed so that
//! future ANN indices (HNSW, FAISS, etc.) can replace the inner loop
//! without changing calling code.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────
// Similarity Result
// ─────────────────────────────────────────────────────────────────

/// A single result from a similarity search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    /// The memory ID of the matched embedding.
    pub memory_id: String,
    /// Cosine similarity score in [-1.0, 1.0].
    pub score: f32,
}

// ─────────────────────────────────────────────────────────────────
// Cosine Similarity
// ─────────────────────────────────────────────────────────────────

/// Compute cosine similarity between two vectors.
///
/// Returns 0.0 for:
/// - mismatched dimensions
/// - zero-magnitude vectors
///
/// The implementation avoids heap allocation and uses a single pass
/// over both vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;

    for (ai, bi) in a.iter().zip(b.iter()) {
        let ai = *ai as f64;
        let bi = *bi as f64;
        dot += ai * bi;
        norm_a += ai * ai;
        norm_b += bi * bi;
    }

    let denom = (norm_a * norm_b).sqrt();
    if denom < 1e-12 {
        return 0.0;
    }

    (dot / denom) as f32
}

// ─────────────────────────────────────────────────────────────────
// Vector Normalization
// ─────────────────────────────────────────────────────────────────

/// Normalize a vector to unit length (L2 normalization) in-place.
///
/// If the vector has zero magnitude, it is left unchanged.
pub fn normalize_vector(v: &mut [f32]) {
    let mag: f64 = v.iter().map(|x| (*x as f64) * (*x as f64)).sum();
    let mag = mag.sqrt();
    if mag < 1e-12 {
        return;
    }
    for x in v.iter_mut() {
        *x = (*x as f64 / mag) as f32;
    }
}

// ─────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_vectors_have_similarity_one() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6, "Expected ~1.0, got {sim}");
    }

    #[test]
    fn orthogonal_vectors_have_similarity_zero() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6, "Expected ~0.0, got {sim}");
    }

    #[test]
    fn opposite_vectors_have_similarity_negative_one() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![-1.0, -2.0, -3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6, "Expected ~-1.0, got {sim}");
    }

    #[test]
    fn zero_vector_returns_zero() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
        assert_eq!(cosine_similarity(&b, &a), 0.0);
    }

    #[test]
    fn mismatched_dimensions_return_zero() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn empty_vectors_return_zero() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn single_element_vectors() {
        let a = vec![3.0];
        let b = vec![5.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6, "Parallel 1D vectors = 1.0");
    }

    #[test]
    fn large_vectors_are_stable() {
        // 1000-dim vectors with small values to test numerical stability
        let a: Vec<f32> = (0..1000).map(|i| (i as f32) * 0.001).collect();
        let b: Vec<f32> = (0..1000).map(|i| (i as f32) * 0.002).collect();
        let sim = cosine_similarity(&a, &b);
        // Both point in the same direction (positive ramp), should be ~1.0
        assert!(sim > 0.99, "Expected high similarity, got {sim}");
    }

    #[test]
    fn normalize_produces_unit_vector() {
        let mut v = vec![3.0, 4.0];
        normalize_vector(&mut v);
        let mag: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (mag - 1.0).abs() < 1e-5,
            "Magnitude should be 1.0, got {mag}"
        );
        assert!((v[0] - 0.6).abs() < 1e-5);
        assert!((v[1] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn normalize_zero_vector_is_unchanged() {
        let mut v = vec![0.0, 0.0, 0.0];
        normalize_vector(&mut v);
        assert_eq!(v, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn normalized_vectors_cosine_equals_dot_product() {
        let mut a = vec![1.0, 2.0, 3.0];
        let mut b = vec![4.0, 5.0, 6.0];
        normalize_vector(&mut a);
        normalize_vector(&mut b);
        let cos = cosine_similarity(&a, &b);
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!(
            (cos - dot).abs() < 1e-5,
            "cos={cos}, dot={dot} — should match for unit vectors"
        );
    }
}
