use serde::{Deserialize, Serialize};

/// Configuration for the compression engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Minimum similarity score (0.0..1.0) to consider items duplicates.
    pub dedup_threshold: f64,
    /// Maximum number of items per cluster.
    pub max_cluster_size: usize,
    /// Minimum frequency to extract a principle.
    pub principle_min_frequency: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            dedup_threshold: 0.8,
            max_cluster_size: 50,
            principle_min_frequency: 3,
        }
    }
}

/// Statistics from a compression run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub input_count: usize,
    pub output_count: usize,
    pub duplicates_removed: usize,
    pub clusters_formed: usize,
    pub principles_extracted: usize,
    pub compression_ratio: f64,
}

/// Result of a compression operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub summaries: Vec<String>,
    pub clusters: Vec<CompressionCluster>,
    pub extracted_principles: Vec<String>,
    pub stats: CompressionStats,
}

/// A cluster formed during compression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionCluster {
    pub id: String,
    pub label: String,
    pub member_ids: Vec<String>,
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_config_default() {
        let cfg = CompressionConfig::default();
        assert!((cfg.dedup_threshold - 0.8).abs() < f64::EPSILON);
        assert_eq!(cfg.max_cluster_size, 50);
        assert_eq!(cfg.principle_min_frequency, 3);
    }

    #[test]
    fn compression_stats_serialization() {
        let stats = CompressionStats {
            input_count: 1000,
            output_count: 100,
            duplicates_removed: 50,
            clusters_formed: 10,
            principles_extracted: 1,
            compression_ratio: 0.1,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let back: CompressionStats = serde_json::from_str(&json).unwrap();
        assert_eq!(back.input_count, 1000);
        assert!((back.compression_ratio - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn compression_result_serialization() {
        let result = CompressionResult {
            summaries: vec!["Summary 1".into()],
            clusters: vec![CompressionCluster {
                id: "c_1".into(),
                label: "Auth cluster".into(),
                member_ids: vec!["m_1".into(), "m_2".into()],
                summary: "Auth-related items".into(),
            }],
            extracted_principles: vec!["Always validate tokens".into()],
            stats: CompressionStats {
                input_count: 100,
                output_count: 10,
                duplicates_removed: 5,
                clusters_formed: 3,
                principles_extracted: 1,
                compression_ratio: 0.1,
            },
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: CompressionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.clusters.len(), 1);
        assert_eq!(back.clusters[0].member_ids.len(), 2);
    }

    #[test]
    fn compression_cluster_serialization() {
        let cluster = CompressionCluster {
            id: "c_1".into(),
            label: "Deploy".into(),
            member_ids: vec!["a".into()],
            summary: "Deployment stuff".into(),
        };
        let json = serde_json::to_string(&cluster).unwrap();
        let back: CompressionCluster = serde_json::from_str(&json).unwrap();
        assert_eq!(back.label, "Deploy");
    }
}
