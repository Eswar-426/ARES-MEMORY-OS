use ares_agent_runtime::models::AgentId;
use std::collections::HashMap;

use super::models::{Conflict, ConflictId, ConflictType, Resolution};
use crate::shared_memory::fact::SharedFact;

/// Resolver for detecting and handling conflicts between agents.
pub struct ConflictResolver {
    conflicts: HashMap<ConflictId, Conflict>,
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            conflicts: HashMap::new(),
        }
    }

    /// Detect conflicts from shared workspace facts.
    pub fn detect_conflicts(&mut self, facts: &[SharedFact]) -> Vec<ConflictId> {
        let mut detected = Vec::new();

        // Detect contradictory facts (same key, different values from different agents)
        let mut key_values: HashMap<&str, Vec<&SharedFact>> = HashMap::new();
        for fact in facts.iter().filter(|f| f.is_active()) {
            key_values.entry(&fact.key).or_default().push(fact);
        }

        for (key, facts_for_key) in &key_values {
            if facts_for_key.len() > 1 {
                let values: Vec<&str> = facts_for_key.iter().map(|f| f.value.as_str()).collect();
                let unique_values: std::collections::HashSet<&str> = values.into_iter().collect();

                if unique_values.len() > 1 {
                    let agents: Vec<AgentId> = facts_for_key.iter().map(|f| f.author).collect();
                    let conflict = Conflict::new(
                        ConflictType::ContradictoryPlans,
                        agents,
                        format!("Contradictory values for key '{}'", key),
                    );
                    let id = conflict.id;
                    self.conflicts.insert(id, conflict);
                    detected.push(id);
                }
            }
        }

        // Detect duplicate work (multiple facts with same category and similar keys from different agents)
        let mut work_items: HashMap<&str, Vec<AgentId>> = HashMap::new();
        for fact in facts
            .iter()
            .filter(|f| f.is_active() && f.key.starts_with("task:"))
        {
            work_items.entry(&fact.key).or_default().push(fact.author);
        }

        for (key, agents) in &work_items {
            let unique_agents: std::collections::HashSet<&AgentId> = agents.iter().collect();
            if unique_agents.len() > 1 {
                let conflict = Conflict::new(
                    ConflictType::DuplicateWork,
                    agents.clone(),
                    format!("Multiple agents working on '{}'", key),
                );
                let id = conflict.id;
                self.conflicts.insert(id, conflict);
                detected.push(id);
            }
        }

        detected
    }

    /// Classify a conflict type from a description.
    pub fn classify_conflict(description: &str) -> ConflictType {
        let desc_lower = description.to_lowercase();
        if desc_lower.contains("resource") || desc_lower.contains("contention") {
            ConflictType::ResourceContention
        } else if desc_lower.contains("contradict") || desc_lower.contains("conflict") {
            ConflictType::ContradictoryPlans
        } else if desc_lower.contains("duplicate") || desc_lower.contains("overlap") {
            ConflictType::DuplicateWork
        } else {
            ConflictType::PriorityClash
        }
    }

    /// Register a manually detected conflict.
    pub fn register_conflict(&mut self, conflict: Conflict) -> ConflictId {
        let id = conflict.id;
        self.conflicts.insert(id, conflict);
        id
    }

    /// Resolve a conflict with a given resolution.
    pub fn resolve(
        &mut self,
        conflict_id: &ConflictId,
        resolution: Resolution,
    ) -> Result<(), String> {
        if let Some(conflict) = self.conflicts.get_mut(conflict_id) {
            conflict.resolve(resolution);
            Ok(())
        } else {
            Err(format!("Conflict {:?} not found", conflict_id))
        }
    }

    /// Get a conflict.
    pub fn get_conflict(&self, conflict_id: &ConflictId) -> Option<&Conflict> {
        self.conflicts.get(conflict_id)
    }

    /// Get all unresolved conflicts.
    pub fn get_unresolved(&self) -> Vec<&Conflict> {
        self.conflicts
            .values()
            .filter(|c| !c.is_resolved())
            .collect()
    }

    /// Get conflict count.
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }

    /// Get resolved conflict count.
    pub fn resolved_count(&self) -> usize {
        self.conflicts.values().filter(|c| c.is_resolved()).count()
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}
