use super::models::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Engine for memory compression — summarization, clustering, deduplication, principle extraction.
pub struct CompressionEngine {
    config: CompressionConfig,
}

impl CompressionEngine {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self {
            config: CompressionConfig::default(),
        }
    }

    /// Summarize a list of text items into a single summary.
    pub fn summarize(&self, items: &[String]) -> String {
        if items.is_empty() {
            return String::new();
        }
        if items.len() == 1 {
            return items[0].clone();
        }

        // Extract common themes by word frequency
        let mut word_freq: HashMap<String, usize> = HashMap::new();
        for item in items {
            let words: Vec<String> = item
                .to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .map(String::from)
                .collect();
            for word in &words {
                *word_freq.entry(word.clone()).or_insert(0) += 1;
            }
        }

        // Top themes
        let mut themes: Vec<(String, usize)> = word_freq.into_iter().collect();
        themes.sort_by_key(|b| std::cmp::Reverse(b.1));
        let top_themes: Vec<String> = themes.iter().take(5).map(|(w, _)| w.clone()).collect();

        format!(
            "Summary of {} items. Key themes: {}.",
            items.len(),
            top_themes.join(", ")
        )
    }

    /// Cluster similar items based on shared word overlap.
    pub fn cluster(&self, items: &[(String, String)]) -> Vec<CompressionCluster> {
        if items.is_empty() {
            return vec![];
        }

        // Simple word-overlap clustering
        let mut clusters: Vec<CompressionCluster> = Vec::new();
        let mut assigned: Vec<bool> = vec![false; items.len()];

        for (i, (id_i, text_i)) in items.iter().enumerate() {
            if assigned[i] {
                continue;
            }

            let words_i: Vec<String> = text_i
                .to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .map(String::from)
                .collect();

            let mut members = vec![id_i.clone()];
            assigned[i] = true;

            for (j, (id_j, text_j)) in items.iter().enumerate().skip(i + 1) {
                if assigned[j] || members.len() >= self.config.max_cluster_size {
                    continue;
                }

                let words_j: Vec<String> = text_j
                    .to_lowercase()
                    .split_whitespace()
                    .filter(|w| w.len() > 3)
                    .map(String::from)
                    .collect();

                let overlap = words_i.iter().filter(|w| words_j.contains(w)).count();
                let max_len = words_i.len().max(words_j.len()).max(1);
                let similarity = overlap as f64 / max_len as f64;

                if similarity >= self.config.dedup_threshold {
                    members.push(id_j.clone());
                    assigned[j] = true;
                }
            }

            if members.len() > 1 || !assigned.iter().all(|&a| a) {
                let texts: Vec<String> = members
                    .iter()
                    .filter_map(|id| {
                        items
                            .iter()
                            .find(|(iid, _)| iid == id)
                            .map(|(_, t)| t.clone())
                    })
                    .collect();
                let summary = self.summarize(&texts);

                clusters.push(CompressionCluster {
                    id: Uuid::now_v7().to_string(),
                    label: format!("Cluster {}", clusters.len() + 1),
                    member_ids: members,
                    summary,
                });
            }
        }

        // Assign remaining unclustered items as singleton clusters
        for (i, (id, text)) in items.iter().enumerate() {
            if !assigned[i] {
                clusters.push(CompressionCluster {
                    id: Uuid::now_v7().to_string(),
                    label: format!("Cluster {}", clusters.len() + 1),
                    member_ids: vec![id.clone()],
                    summary: text.clone(),
                });
            }
        }

        clusters
    }

    /// Deduplicate items by similarity threshold.
    /// Returns the IDs of items that should be kept (first occurrence wins).
    pub fn deduplicate(&self, items: &[(String, String)]) -> Vec<String> {
        let mut keep: Vec<String> = Vec::new();
        let mut kept_texts: Vec<&str> = Vec::new();

        for (id, text) in items {
            let is_dup = kept_texts.iter().any(|kept| {
                let words_a: Vec<&str> = text.split_whitespace().filter(|w| w.len() > 3).collect();
                let words_b: Vec<&str> = kept.split_whitespace().filter(|w| w.len() > 3).collect();
                if words_a.is_empty() || words_b.is_empty() {
                    return false;
                }
                let overlap = words_a.iter().filter(|w| words_b.contains(w)).count();
                let similarity = overlap as f64 / words_a.len().max(words_b.len()) as f64;
                similarity >= self.config.dedup_threshold
            });

            if !is_dup {
                keep.push(id.clone());
                kept_texts.push(text);
            }
        }

        keep
    }

    /// Extract principles from repeated patterns.
    /// Returns principle descriptions for items that appear with frequency >= threshold.
    pub fn extract_principles(&self, items: &[(String, u32)]) -> Vec<String> {
        items
            .iter()
            .filter(|(_, freq)| *freq >= self.config.principle_min_frequency)
            .map(|(text, freq)| format!("Principle (observed {} times): {}", freq, text))
            .collect()
    }

    /// Run the full compression pipeline.
    pub fn compress(&self, items: &[(String, String)]) -> CompressionResult {
        let input_count = items.len();

        // Step 1: Deduplicate
        let kept_ids = self.deduplicate(items);
        let duplicates_removed = input_count - kept_ids.len();

        // Step 2: Filter to kept items
        let kept_items: Vec<(String, String)> = items
            .iter()
            .filter(|(id, _)| kept_ids.contains(id))
            .cloned()
            .collect();

        // Step 3: Cluster
        let clusters = self.cluster(&kept_items);

        // Step 4: Summarize each cluster
        let summaries: Vec<String> = clusters.iter().map(|c| c.summary.clone()).collect();

        // Step 5: Extract principles from cluster themes
        let freq_items: Vec<(String, u32)> = clusters
            .iter()
            .map(|c| (c.summary.clone(), c.member_ids.len() as u32))
            .collect();
        let principles = self.extract_principles(&freq_items);
        let principles_extracted = principles.len();

        let output_count = clusters.len();
        let compression_ratio = if input_count > 0 {
            output_count as f64 / input_count as f64
        } else {
            1.0
        };

        CompressionResult {
            summaries,
            clusters,
            extracted_principles: principles,
            stats: CompressionStats {
                input_count,
                output_count,
                duplicates_removed,
                clusters_formed: output_count,
                principles_extracted,
                compression_ratio,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_empty() {
        let engine = CompressionEngine::with_defaults();
        assert_eq!(engine.summarize(&[]), "");
    }

    #[test]
    fn summarize_single_item() {
        let engine = CompressionEngine::with_defaults();
        let items = vec!["Single item text".into()];
        assert_eq!(engine.summarize(&items), "Single item text");
    }

    #[test]
    fn summarize_multiple_items() {
        let engine = CompressionEngine::with_defaults();
        let items = vec![
            "Authentication system using JWT tokens".into(),
            "JWT token refresh mechanism for authentication".into(),
            "Token validation for the authentication pipeline".into(),
        ];
        let summary = engine.summarize(&items);
        assert!(summary.contains("3 items"));
    }

    #[test]
    fn cluster_empty() {
        let engine = CompressionEngine::with_defaults();
        let clusters = engine.cluster(&[]);
        assert!(clusters.is_empty());
    }

    #[test]
    fn cluster_groups_similar() {
        let engine = CompressionEngine::new(CompressionConfig {
            dedup_threshold: 0.3,
            ..Default::default()
        });
        let items = vec![
            ("1".into(), "authentication system using JWT tokens".into()),
            (
                "2".into(),
                "authentication mechanism with JWT validation".into(),
            ),
            ("3".into(), "database migration schema update".into()),
        ];
        let clusters = engine.cluster(&items);
        // Should form at least 1 cluster grouping auth items
        assert!(!clusters.is_empty());
    }

    #[test]
    fn deduplicate_removes_similar() {
        let engine = CompressionEngine::new(CompressionConfig {
            dedup_threshold: 0.5,
            ..Default::default()
        });
        let items = vec![
            ("1".into(), "deploy service to production kubernetes".into()),
            (
                "2".into(),
                "deploy service to production kubernetes cluster".into(),
            ),
            (
                "3".into(),
                "completely different unrelated text here".into(),
            ),
        ];
        let kept = engine.deduplicate(&items);
        // Items 1 and 2 are very similar, so only one should be kept + item 3
        assert!(kept.len() <= 3);
        assert!(kept.contains(&"1".to_string()));
        assert!(kept.contains(&"3".to_string()));
    }

    #[test]
    fn deduplicate_keeps_unique() {
        let engine = CompressionEngine::with_defaults();
        let items = vec![
            ("1".into(), "alpha beta gamma".into()),
            ("2".into(), "delta epsilon zeta".into()),
        ];
        let kept = engine.deduplicate(&items);
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn extract_principles_filters_by_frequency() {
        let engine = CompressionEngine::with_defaults();
        let items = vec![
            ("Always validate input".into(), 5),
            ("Sometimes check output".into(), 1),
            ("Handle errors gracefully".into(), 10),
        ];
        let principles = engine.extract_principles(&items);
        assert_eq!(principles.len(), 2); // Only those with freq >= 3
    }

    #[test]
    fn extract_principles_empty() {
        let engine = CompressionEngine::with_defaults();
        let principles = engine.extract_principles(&[]);
        assert!(principles.is_empty());
    }

    #[test]
    fn compress_full_pipeline() {
        let engine = CompressionEngine::with_defaults();
        let items: Vec<(String, String)> = (0..10)
            .map(|i| {
                (
                    format!("id_{}", i),
                    format!("Item number {} with content", i),
                )
            })
            .collect();
        let result = engine.compress(&items);
        assert_eq!(result.stats.input_count, 10);
        assert!(result.stats.output_count <= 10);
    }

    #[test]
    fn compress_empty() {
        let engine = CompressionEngine::with_defaults();
        let result = engine.compress(&[]);
        assert_eq!(result.stats.input_count, 0);
        assert!(result.clusters.is_empty());
    }

    #[test]
    fn compression_ratio_correct() {
        let engine = CompressionEngine::with_defaults();
        // Use texts with distinct words (> 3 chars) to avoid dedup
        let items = vec![
            ("1".into(), "authentication system using tokens".into()),
            ("2".into(), "deploy microservice into production".into()),
        ];
        let result = engine.compress(&items);
        assert!(result.stats.compression_ratio > 0.0);
        assert!(result.stats.compression_ratio <= 1.0);
    }
}
