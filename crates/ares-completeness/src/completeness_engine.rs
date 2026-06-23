use crate::models::{HierarchySegment, TopologyState};
use ares_reasoning::models::MemoryGap;

pub struct CompletenessEngine;

impl Default for CompletenessEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletenessEngine {
    pub fn new() -> Self {
        Self
    }

    /// Converts partial and orphaned topology states into MemoryGaps representing missing downstream paths.
    pub fn find_completeness_gaps(&self, segments: &[HierarchySegment]) -> Vec<MemoryGap> {
        let mut gaps = Vec::new();

        for seg in segments {
            if seg.state == TopologyState::Orphaned {
                gaps.push(MemoryGap {
                    from_type: seg.node_type.clone(),
                    to_type: "Any".to_string(),
                    node_id: seg.node_id.clone(),
                    gap_description: format!(
                        "Node '{}' of type {} is orphaned (disconnected from all hierarchy)",
                        seg.node_id, seg.node_type
                    ),
                    confidence: 1.0,
                });
            } else if seg.state == TopologyState::Partial {
                let missing_str = seg.missing_downstream.join(", ");
                gaps.push(MemoryGap {
                    from_type: seg.node_type.clone(),
                    to_type: seg
                        .missing_downstream
                        .first()
                        .unwrap_or(&"Unknown".to_string())
                        .clone(),
                    node_id: seg.node_id.clone(),
                    gap_description: format!(
                        "Node '{}' of type {} is partially traced. Missing downstream: {}",
                        seg.node_id, seg.node_type, missing_str
                    ),
                    confidence: 1.0,
                });
            }
        }

        gaps
    }

    pub fn calculate_completeness_score(&self, segments: &[HierarchySegment]) -> f32 {
        if segments.is_empty() {
            return 0.0;
        }

        let mut complete = 0;
        let total = segments.len();

        for seg in segments {
            if seg.state == TopologyState::Complete {
                complete += 1;
            } else if seg.state == TopologyState::Partial {
                // Partial credit? Let's just do binary for completeness: either full chain or not.
                // Or maybe weighted by missing parts? For now, binary.
            }
        }

        (complete as f32 / total as f32) * 100.0
    }
}
