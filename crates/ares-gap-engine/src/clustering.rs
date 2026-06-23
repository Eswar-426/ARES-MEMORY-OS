use crate::models::{Gap, GapCluster, GapSeverity, RootCause};
use ares_core::id::new_id;
use std::collections::HashMap;

pub struct GapClusterEngine;

impl Default for GapClusterEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GapClusterEngine {
    pub fn new() -> Self {
        Self
    }

    /// Groups individual gaps into systemic clusters based on RootCause.
    pub fn cluster(&self, gaps: &[Gap]) -> Vec<GapCluster> {
        let mut grouped: HashMap<RootCause, Vec<&Gap>> = HashMap::new();

        for gap in gaps {
            if let Some(reason) = &gap.reason {
                grouped
                    .entry(reason.root_cause.clone())
                    .or_default()
                    .push(gap);
            }
        }

        let mut clusters = Vec::new();

        for (cause, cluster_gaps) in grouped {
            let mut affected_entities = Vec::new();
            let mut max_severity = GapSeverity::Info;

            for gap in &cluster_gaps {
                affected_entities.push(gap.source_id.clone());

                // Track highest severity in cluster
                if gap.severity == GapSeverity::Critical {
                    max_severity = GapSeverity::Critical;
                } else if gap.severity == GapSeverity::Warning && max_severity == GapSeverity::Info
                {
                    max_severity = GapSeverity::Warning;
                }
            }

            // Deduplicate entities
            affected_entities.sort();
            affected_entities.dedup();

            let summary = format!(
                "Identified {} instances of {:?}. This systemic issue affects {} entities.",
                cluster_gaps.len(),
                cause,
                affected_entities.len()
            );

            clusters.push(GapCluster {
                id: format!(
                    "cluster_{}_{}",
                    format!("{:?}", cause).to_lowercase(),
                    new_id()
                ),
                root_cause: cause,
                affected_entities,
                severity: max_severity,
                summary,
            });
        }

        clusters
    }
}
