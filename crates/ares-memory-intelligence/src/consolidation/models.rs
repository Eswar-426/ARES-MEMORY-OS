use serde::{Deserialize, Serialize};

/// Type of memory cluster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterType {
    Topic,
    Temporal,
    Causal,
    Similarity,
}

impl ClusterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Topic => "topic",
            Self::Temporal => "temporal",
            Self::Causal => "causal",
            Self::Similarity => "similarity",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "topic" => Self::Topic,
            "temporal" => Self::Temporal,
            "causal" => Self::Causal,
            "similarity" => Self::Similarity,
            _ => Self::Topic,
        }
    }
}

/// A cluster of related memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCluster {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cluster_type: ClusterType,
    pub member_count: u32,
    pub centroid_tags: Vec<String>,
    pub summary: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Association between a cluster and an episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMembership {
    pub cluster_id: String,
    pub episode_id: String,
    pub similarity: f64,
    pub added_at: i64,
}

/// Configuration for consolidation.
#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    /// Minimum similarity to consider episodes duplicates.
    pub merge_threshold: f64,
    /// Minimum cluster size to generate a summary.
    pub min_cluster_size: usize,
    /// Maximum age in days before archiving low-value events.
    pub archive_age_days: u32,
    /// Minimum score to keep (below this gets archived).
    pub archive_min_score: f64,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            merge_threshold: 0.7,
            min_cluster_size: 3,
            archive_age_days: 90,
            archive_min_score: 0.3,
        }
    }
}

/// Result of a consolidation cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub duplicates_merged: u32,
    pub clusters_formed: u32,
    pub patterns_detected: u32,
    pub summaries_generated: u32,
    pub events_archived: u32,
}

/// A detected recurring pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringPattern {
    pub description: String,
    pub frequency: u32,
    pub episode_ids: Vec<String>,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_type_roundtrip() {
        for t in &[
            ClusterType::Topic,
            ClusterType::Temporal,
            ClusterType::Causal,
            ClusterType::Similarity,
        ] {
            assert_eq!(&ClusterType::from_str_val(t.as_str()), t);
        }
    }

    #[test]
    fn memory_cluster_serialization() {
        let cluster = MemoryCluster {
            id: "cl_1".into(),
            name: "Auth cluster".into(),
            description: "Authentication-related episodes".into(),
            cluster_type: ClusterType::Topic,
            member_count: 5,
            centroid_tags: vec!["auth".into(), "jwt".into()],
            summary: "Multiple auth episodes".into(),
            created_at: 1000,
            updated_at: 2000,
        };
        let json = serde_json::to_string(&cluster).unwrap();
        let back: MemoryCluster = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Auth cluster");
        assert_eq!(back.centroid_tags.len(), 2);
    }

    #[test]
    fn consolidation_config_default() {
        let cfg = ConsolidationConfig::default();
        assert!((cfg.merge_threshold - 0.7).abs() < f64::EPSILON);
        assert_eq!(cfg.min_cluster_size, 3);
    }

    #[test]
    fn consolidation_result_serialization() {
        let result = ConsolidationResult {
            duplicates_merged: 5,
            clusters_formed: 3,
            patterns_detected: 2,
            summaries_generated: 3,
            events_archived: 10,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ConsolidationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.duplicates_merged, 5);
    }

    #[test]
    fn recurring_pattern_serialization() {
        let pattern = RecurringPattern {
            description: "Timeout during deploy".into(),
            frequency: 7,
            episode_ids: vec!["ep_1".into(), "ep_2".into()],
            confidence: 0.9,
        };
        let json = serde_json::to_string(&pattern).unwrap();
        let back: RecurringPattern = serde_json::from_str(&json).unwrap();
        assert_eq!(back.frequency, 7);
    }

    #[test]
    fn cluster_membership_serialization() {
        let membership = ClusterMembership {
            cluster_id: "cl_1".into(),
            episode_id: "ep_1".into(),
            similarity: 0.85,
            added_at: 1000,
        };
        let json = serde_json::to_string(&membership).unwrap();
        let back: ClusterMembership = serde_json::from_str(&json).unwrap();
        assert!((back.similarity - 0.85).abs() < f64::EPSILON);
    }
}
